<template>
  <TextField label="Duration" v-model="duration" />
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import TextField from '../core/TextField.vue';

export default defineComponent({
  name: 'TSESettings',
  components: { TextField },
  props: {
    modelValue: {
      type: String,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  computed: {
    duration: {
      get(): string {
        return this.modelValue ?? '';
      },
      set(v: string) {
        this.$emit('update:modelValue', v);
      },
    },
  },
});
</script>
