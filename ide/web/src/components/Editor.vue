<template>
  <div class="editor-container" ref="editorElement"></div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from "vue";
import * as monaco from "monaco-editor";

const emit = defineEmits({
  "update:modelValue": (value: string) => true,
});
const props = defineProps<{
  options?: monaco.editor.IStandaloneEditorConstructionOptions;
  modelValue?: string;
}>();

const editorElement = ref(undefined as any);
let editor = undefined as monaco.editor.IStandaloneCodeEditor | undefined;

const disposables: Array<monaco.IDisposable> = [];

watch(
  () => props.modelValue,
  val => {
    if (typeof val === "undefined") {
      return;
    }

    if (editor?.getValue() !== val) {
      editor?.setValue(val);
    }
  },
);

watch(editorElement, el => {
  if (editor) {
    return;
  }

  if (el instanceof HTMLElement) {
    editor = monaco.editor.create(el, {
      value: props.modelValue ?? "",
      language: "json",
      lineNumbers: "on",
      scrollBeyondLastLine: true,
      readOnly: false,
      theme: "vs-dark",
      letterSpacing: 1,
      useTabStops: true,
      minimap: {
        enabled: true,
      },
      automaticLayout: true,
      fixedOverflowWidgets: true,
      glyphMargin: true,
    });

    disposables.push(
      editor.getModel()!.onDidChangeContent(_ => {
        const val = editor?.getValue();

        if (typeof val !== "undefined" && props.modelValue !== val) {
          emit("update:modelValue", val);
        }
      }),
    );
  } else {
    throw new Error("invalid editor element");
  }
});

onUnmounted(() => {
  if (editor) {
    editor.dispose();
  }

  for (const d of disposables) {
    d.dispose();
  }
});
</script>

<style lang="scss">
.editor-container {
  height: 100%;
  width: 100%;
  padding: 0;
  margin: 0;
}
</style>
