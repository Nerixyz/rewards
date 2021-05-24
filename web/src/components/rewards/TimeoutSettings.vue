<template>
<TextField label='Duration' v-model='duration' />
</template>

<script lang='ts'>
import { defineComponent } from 'vue';
import TextField from '../core/TextField.vue';

export default defineComponent({
  name: 'TimeoutSettings',
  components: { TextField },
  props: {
    modelValue: String
  },
  emits: ['update:modelValue'],
  computed: {
    duration: {
      get() {
        return this.modelValue;
      },
      set(v: string) {
        this.$emit('update:modelValue', v);
      }
    }
  }
});
</script>
