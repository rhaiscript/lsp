use quote::{quote, ToTokens};
use syn::{parse::Parse, parse::ParseStream};

use crate::function::ExportedFn;
use crate::rhai_module::ExportedConst;

#[cfg(no_std)]
use alloc::vec as new_vec;
#[cfg(not(no_std))]
use std::vec as new_vec;

#[cfg(no_std)]
use core::mem;
#[cfg(not(no_std))]
use std::mem;

use std::borrow::Cow;

use crate::attrs::{AttrItem, ExportInfo, ExportScope, ExportedParams};
use crate::function::ExportedFnParams;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct ExportedModParams {
    pub name: String,
    skip: bool,
    pub scope: ExportScope,
}

impl Parse for ExportedModParams {
    fn parse(args: ParseStream) -> syn::Result<Self> {
        if args.is_empty() {
            return Ok(ExportedModParams::default());
        }

        let info = crate::attrs::parse_attr_items(args)?;

        Self::from_info(info)
    }
}

impl ExportedParams for ExportedModParams {
    fn parse_stream(args: ParseStream) -> syn::Result<Self> {
        Self::parse(args)
    }

    fn no_attrs() -> Self {
        Default::default()
    }

    fn from_info(info: ExportInfo) -> syn::Result<Self> {
        let ExportInfo { items: attrs, .. } = info;
        let mut name = Default::default();
        let mut skip = false;
        let mut scope = None;
        for attr in attrs {
            let AttrItem { key, value, .. } = attr;
            match (key.to_string().as_ref(), value) {
                ("name", Some(s)) => {
                    let new_name = s.value();
                    if name == new_name {
                        return Err(syn::Error::new(key.span(), "conflicting name"));
                    }
                    name = new_name;
                }
                ("name", None) => return Err(syn::Error::new(key.span(), "requires value")),

                ("skip", None) => skip = true,
                ("skip", Some(s)) => return Err(syn::Error::new(s.span(), "extraneous value")),

                ("export_prefix", Some(_)) | ("export_all", None) if scope.is_some() => {
                    return Err(syn::Error::new(key.span(), "duplicate export scope"));
                }
                ("export_prefix", Some(s)) => scope = Some(ExportScope::Prefix(s.value())),
                ("export_prefix", None) => {
                    return Err(syn::Error::new(key.span(), "requires value"))
                }
                ("export_all", None) => scope = Some(ExportScope::All),
                ("export_all", Some(s)) => {
                    return Err(syn::Error::new(s.span(), "extraneous value"))
                }
                (attr, _) => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown attribute '{}'", attr),
                    ))
                }
            }
        }

        Ok(ExportedModParams {
            name,
            skip,
            scope: scope.unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
pub struct Module {
    mod_all: syn::ItemMod,
    fns: Vec<ExportedFn>,
    consts: Vec<ExportedConst>,
    sub_modules: Vec<Module>,
    params: ExportedModParams,
}

impl Module {
    pub fn set_params(&mut self, params: ExportedModParams) -> syn::Result<()> {
        self.params = params;
        Ok(())
    }
}

impl Parse for Module {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut mod_all: syn::ItemMod = input.parse()?;
        let fns: Vec<_>;
        let mut consts: Vec<_> = new_vec![];
        let mut sub_modules: Vec<_> = Vec::new();
        if let Some((_, ref mut content)) = mod_all.content {
            // Gather and parse functions.
            fns = content
                .iter_mut()
                .filter_map(|item| match item {
                    syn::Item::Fn(f) => Some(f),
                    _ => None,
                })
                .try_fold(Vec::new(), |mut vec, item_fn| {
                    // #[cfg] attributes are not allowed on functions
                    crate::attrs::deny_cfg_attr(&item_fn.attrs)?;

                    let params: ExportedFnParams =
                        match crate::attrs::inner_item_attributes(&mut item_fn.attrs, "rhai_fn") {
                            Ok(p) => p,
                            Err(e) => return Err(e),
                        };
                    syn::parse2::<ExportedFn>(item_fn.to_token_stream())
                        .and_then(|mut f| {
                            f.set_params(params)?;
                            Ok(f)
                        })
                        .map(|f| vec.push(f))
                        .map(|_| vec)
                })?;
            // Gather and parse constants definitions.
            for item in content.iter() {
                match item {
                    syn::Item::Const(syn::ItemConst {
                        vis,
                        ref expr,
                        ident,
                        attrs,
                        ty,
                        ..
                    }) => {
                        // #[cfg] attributes are not allowed on const declarations
                        crate::attrs::deny_cfg_attr(&attrs)?;
                        match vis {
                            syn::Visibility::Public(_) => {
                                consts.push((ident.to_string(), ty.clone(), expr.as_ref().clone()))
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            // Gather and parse sub-module definitions.
            //
            // They are actually removed from the module's body, because they will need
            // re-generating later when generated code is added.
            sub_modules.reserve(content.len() - fns.len() - consts.len());
            let mut i = 0;
            while i < content.len() {
                match content[i] {
                    syn::Item::Mod(_) => {
                        let mut item_mod = match content.remove(i) {
                            syn::Item::Mod(m) => m,
                            _ => unreachable!(),
                        };
                        let params: ExportedModParams =
                            crate::attrs::inner_item_attributes(&mut item_mod.attrs, "rhai_mod")?;
                        let module = syn::parse2::<Module>(item_mod.to_token_stream()).and_then(
                            |mut m| {
                                m.set_params(params)?;
                                Ok(m)
                            },
                        )?;
                        sub_modules.push(module);
                    }
                    _ => i += 1,
                }
            }
        } else {
            fns = new_vec![];
        }
        Ok(Module {
            mod_all,
            fns,
            consts,
            sub_modules,
            params: ExportedModParams::default(),
        })
    }
}

impl Module {
    pub fn attrs(&self) -> &Vec<syn::Attribute> {
        &self.mod_all.attrs
    }

    pub fn module_name(&self) -> &syn::Ident {
        &self.mod_all.ident
    }

    pub fn exported_name(&self) -> Cow<str> {
        if !self.params.name.is_empty() {
            (&self.params.name).into()
        } else {
            self.module_name().to_string().into()
        }
    }

    pub fn update_scope(&mut self, parent_scope: &ExportScope) {
        let keep = match (self.params.skip, parent_scope) {
            (true, _) => false,
            (_, ExportScope::PubOnly) => matches!(self.mod_all.vis, syn::Visibility::Public(_)),
            (_, ExportScope::Prefix(s)) => self.mod_all.ident.to_string().starts_with(s),
            (_, ExportScope::All) => true,
        };
        self.params.skip = !keep;
    }

    pub fn skipped(&self) -> bool {
        self.params.skip
    }

    pub fn generate(self) -> proc_macro2::TokenStream {
        match self.generate_inner() {
            Ok(tokens) => tokens,
            Err(e) => e.to_compile_error(),
        }
    }

    fn generate_inner(self) -> Result<proc_macro2::TokenStream, syn::Error> {
        // Check for collisions if the "name" attribute was used on inner functions.
        crate::rhai_module::check_rename_collisions(&self.fns)?;

        // Extract the current structure of the module.
        let Module {
            mut mod_all,
            mut fns,
            consts,
            mut sub_modules,
            params,
            ..
        } = self;
        let mod_vis = mod_all.vis;
        let mod_name = mod_all.ident.clone();
        let (_, orig_content) = mod_all.content.take().unwrap();
        let mod_attrs = mem::take(&mut mod_all.attrs);

        if !params.skip {
            // Generate new module items.
            //
            // This is done before inner module recursive generation, because that is destructive.
            let mod_gen = crate::rhai_module::generate_body(
                &mut fns,
                &consts,
                &mut sub_modules,
                &params.scope,
            );

            // NB: sub-modules must have their new items for exporting generated in depth-first order
            // to avoid issues caused by re-parsing them
            let inner_modules: Vec<proc_macro2::TokenStream> = sub_modules.drain(..)
                .try_fold::<Vec<proc_macro2::TokenStream>, _,
                            Result<Vec<proc_macro2::TokenStream>, syn::Error>>(
                    Vec::new(), |mut acc, m| { acc.push(m.generate_inner()?); Ok(acc) })?;

            // Regenerate the module with the new content added.
            Ok(quote! {
                #(#mod_attrs)*
                #mod_vis mod #mod_name {
                    #(#orig_content)*
                    #(#inner_modules)*
                    #mod_gen
                }
            })
        } else {
            // Regenerate the original module as-is.
            Ok(quote! {
                #(#mod_attrs)*
                #mod_vis mod #mod_name {
                    #(#orig_content)*
                }
            })
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &syn::Ident {
        &self.mod_all.ident
    }

    #[allow(dead_code)]
    pub fn consts(&self) -> &[ExportedConst] {
        &self.consts
    }

    #[allow(dead_code)]
    pub fn fns(&self) -> &[ExportedFn] {
        &self.fns
    }

    #[allow(dead_code)]
    pub fn sub_modules(&self) -> &[Module] {
        &self.sub_modules
    }

    #[allow(dead_code)]
    pub fn content(&self) -> Option<&Vec<syn::Item>> {
        match self.mod_all {
            syn::ItemMod {
                content: Some((_, ref vec)),
                ..
            } => Some(vec),
            _ => None,
        }
    }
}
