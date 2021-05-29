<template>
  <div class="px-20 pt-5 xl:max-w-7xl mx-auto">
    <div class="flex gap-5 flex-col">
      <form class="w-full min-w-10rem border-b border-gray-900 border-opacity-20 pb-6" @submit="addEditor">
        <h3 class="ml-1 mb-3 font-serif text-xl">Add an editor</h3>
        <div class="flex w-full items-center">
          <TextField v-model="editorAddName" class="flex-grow" label="Name" :disabled="loading" />
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

      <div v-if="loading">Loading...</div>
      <div v-else-if="error">
        Failed!
        <br />
        <pre>{{ error }}</pre>
        <OutlinedButton>Ok</OutlinedButton>
      </div>
      <div v-else class="">
        <div v-if="!editors.length">
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
            v-for="editor of editors"
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
import { asyncRefs, tryAsync } from '../utilities';
import TextField from '../components/core/TextField.vue';
import OutlinedButton from '../components/core/OutlinedButton.vue';
import { TwitchUser } from '../api/types';
import PlaneIcon from '../components/icons/PlaneIcon.vue';

export default defineComponent({
  name: 'EditorsDashboard',
  components: { PlaneIcon, TextField, OutlinedButton },

  setup() {
    const api = useApi();

    const { error, loading } = asyncRefs();
    const editors = ref<TwitchUser[]>([]);
    tryAsync(
      async () => {
        editors.value = await api.getEditors();
      },
      loading,
      error,
    );

    const editorAddName = ref('');

    const addEditor = async (e: Event) => {
      e.preventDefault();
      try {
        const name = editorAddName.value.toLowerCase();
        loading.value = true;
        await api.addEditor(name);
        const user = await api.getUserInfo(name);
        editors.value = [...editors.value, user];
        editorAddName.value = '';
      } catch (e) {
        error.value = e.message;
      } finally {
        loading.value = false;
      }
    };
    const removeEditor = async (name: string) => {
      try {
        loading.value = true;
        await api.removeEditor(name);
        editors.value = editors.value.filter(x => x.login !== name);
      } catch (e) {
        error.value = e.message;
      } finally {
        loading.value = false;
      }
    };

    return { loading, error, addEditor, removeEditor, editorAddName, editors };
  },
});
</script>
