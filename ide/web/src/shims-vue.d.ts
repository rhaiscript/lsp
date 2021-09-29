declare module "*.vue" {
  import { DefineComponent } from "vue";
  const component: DefineComponent<Record<string, any>, Record<string, any>, any>;
  export default component;
}

declare module "*.yaml" {
  const data: any;
  export default data;
}
