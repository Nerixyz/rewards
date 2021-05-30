<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <div class="flex gap-5 flex-col">
      <form class="w-full min-w-10rem border-b border-gray-900 border-opacity-20 pb-6" @submit="addEditor">
        <h3 class="ml-1 mb-3 font-serif text-xl">Add an editor</h3>
        <div class="flex w-full items-center">
          <TextField v-model="editorAddName" class="flex-grow" label="Name" :disabled="state.loading" />
          <button
            type="submit"
            class="
              mx-3
              px-3
              py-3
              rounded-full
              bg-gray-350 bg-opacity-40
              hover:bg-gray-500 hover:bg-opacity-40
              transition transition-color
            "
          >
            <PlaneIcon />
          </button>
        </div>
      </form>

      <div v-if="state.loading"><CLoader/></div>
      <div v-else-if="state.error">
        Failed!
        <br />
        <span class="break-words font-mono">{{ state.error }}</span>
        <OutlinedButton>Ok</OutlinedButton>
      </div>
      <div v-else class="">
        <div v-if="!state.value.length">
          No editors? Seriously? Let your mods do the work, add some editors!
          <img
            class="inline w-5 h-auto mr-1"
            src="https://cdn.betterttv.net/emote/58ca80db994bb43c8d2ffa96/2x"
            alt="FeelsGladMan"
          />
          <img
            class="inline w-5 h-auto"
            src="https://cdn.betterttv.net/emote/5e1b12868af14b5f1b43921d/2x"
            alt="ModTime"
          />
        </div>
        <div v-else class="flex flex-wrap gap-5">
          <div
            v-for="editor of state.value"
            :key="editor.id"
            class="
              flex
              items-center
              flex-col
              gap-4
              bg-gray-300
              rounded-lg
              overflow-hidden
              border border-gray-900 border-opacity-30
              pt-4
              select-none
            "
            @click="removeEditor(editor.login)"
          >
            <img
              :src="editor.profile_image_url"
              :alt="`Profile image of ${editor.login}`"
              class="w-10 h-10 rounded-full"
            />
            <h3>{{ editor.login }}</h3>
            <button
              class="
                bg-red
                text-black
                font-bold
                uppercase
                w-full
                px-7
                py-2
                transition transition-colors
                hover:bg-transparent
                hover:text-red
                focus:bg-transparent
                focus:text-red
                focus:outline-none
              "
              @click="removeEditor(editor.login)"
            >
              Remove
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue';
import { useApi } from '../api/plugin';
import TextField from '../components/core/TextField.vue';
import OutlinedButton from '../components/core/OutlinedButton.vue';
import { TwitchUser } from '../api/types';
import PlaneIcon from '../components/icons/PlaneIcon.vue';
import { asyncState, tryAsync } from '../async-state';
import CLoader from '../components/core/CLoader.vue';

export default defineComponent({
  name: 'EditorsDashboard',
  components: { CLoader, PlaneIcon, TextField, OutlinedButton },

  setup() {
    const api = useApi();

    const { state } = asyncState<TwitchUser[]>([]);
    tryAsync(async state => {
      state.value = await api.getEditors();
    }, state);

    const editorAddName = ref('');

    const addEditor = (e: Event) => {
      e.preventDefault();
      tryAsync(async state => {
        const name = editorAddName.value.toLowerCase();
        await api.addEditor(name);
        const user = await api.getUserInfo(name);
        state.value = [...state.value, user];
        editorAddName.value = '';
      }, state);
    };
    const removeEditor = (name: string) => {
      tryAsync(async state => {
        await api.removeEditor(name);
        state.value = state.value.filter(x => x.login !== name);
      }, state);
    };

    return { state, addEditor, removeEditor, editorAddName };
  },
});
</script>
