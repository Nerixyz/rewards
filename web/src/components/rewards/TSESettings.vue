<template>
  <TextField :model-value="modelValue" label="Duration" :warn="warn" @update:model-value="onUpdate" />
</template>

<script lang="ts">
import { computed, defineComponent, toRefs, watch } from 'vue';
import TextField from '../core/TextField.vue';
import { isValidRewardDurationExpression } from '../../utilities';

export default defineComponent({
  name: 'TSESettings',
  components: { TextField },
  props: {
    modelValue: {
      type: String,
      required: true,
    },
  },
  emits: ['update:modelValue', 'update:warn'],
  setup(props, { emit }) {
    const { modelValue } = toRefs(props);

    const warn = computed(() => !isValidRewardDurationExpression(modelValue.value));
    const onUpdate = (value: string) => {
      emit('update:modelValue', value);
    };
    watch(warn, v => emit('update:warn', v));

    return { onUpdate, modelValue, warn };
  },
});
</script>
