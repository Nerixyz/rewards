<template>
  <TransitionRoot :show="open" as="template">
    <Dialog as='div'>
      <div class="fixed inset-0 z-10 overflow-y-auto">
        <div class="min-h-screen px-4 text-center">
          <TransitionChild
            as="div"
            enter="duration-300 ease-out"
            enter-from="opacity-0"
            enter-to="opacity-100"
            leave="duration-200 ease-in"
            leave-from="opacity-100"
            leave-to="opacity-0"
          >
            <DialogOverlay class="fixed inset-0 bg-black opacity-50" />
          </TransitionChild>

          <span class="inline-block h-screen align-middle" aria-hidden="true">
            <!-- zero-width space -->
            &#x200b;
          </span>

          <TransitionChild
            as="template"
            enter="duration-300 ease-hyper-out"
            enter-from="opacity-0 scale-75"
            enter-to="opacity-100 scale-100"
            leave="duration-200 ease-hyper-in"
            leave-from="opacity-100 scale-100"
            leave-to="opacity-0 scale-75"
            @after-leave='onDialogClose'
          >
            <div
              class="
                inline-block
                p-6
                my-8
                overflow-hidden
                text-left
                align-middle
                transition-all
                transform
                bg-gray-300
                border-4
                border-red
                shadow-xl
                rounded-2xl
                text-white
              "
            >
              <DialogTitle as="h3" class="text-2xl mb-3 font-serif font-medium leading-6 text-white">
                {{ title }}
              </DialogTitle>
              <DialogDescription v-if='subtitle' class='mb-4 text-gray-100 font-serif'>
                {{subtitle}}
              </DialogDescription>

              <slot/>
            </div>
          </TransitionChild>
        </div>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import { TransitionRoot, Dialog, TransitionChild, DialogDescription, DialogOverlay, DialogTitle } from '@headlessui/vue';

export default defineComponent({
  name: 'CDialog',
  props: {
    open: {
      type: Boolean,
      required: true,
    },
    title: {
      type: String,
      required: true,
    },
    subtitle: {
      type: String,
      required: false,
    },
  },
  emits: ['update:open', 'dialogClosed'],
  components: { TransitionRoot, Dialog, TransitionChild, DialogDescription, DialogOverlay, DialogTitle },
  methods: {
    onDialogClose() {
      this.$emit('dialogClosed', true);
    }
  },
});
</script>
