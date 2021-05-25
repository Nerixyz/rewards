<template>
  <div class="px-20 pt-5">
    <div v-if="loading">Loading...</div>
    <div v-else-if="error">
      Failed!
      <br />
      <pre>{{ error }}</pre>
    </div>

    <div v-else class="flex gap-10">
      <form @submit="addEditor" class="w-1/6 min-w-10rem">
        <h3 class="mb-3">Add an editor</h3>
        <TextField label='Name' v-model="editorAddName" :disabled="loading" />
      </form>
      <div class="flex-grow">
        <div v-if='!editors.length'>
          No editors? Seriously? Let your mods do the work, add some editors!
          <img class='inline w-5 h-auto mr-1' src='https://cdn.betterttv.net/emote/58ca80db994bb43c8d2ffa96/2x' alt='FeelsGladMan'/>
          <img class='inline w-5 h-auto' src='https://cdn.betterttv.net/emote/5e1b12868af14b5f1b43921d/2x' alt='ModTime' />
        </div>
        <div v-else v-for="editor of editors" :key="editor.id" class="flex items-center gap-4">
          <img
            :src="editor.profile_image_url"
            :alt="`Profile image of ${editor.login}`"
            class="w-10 h-10 rounded-full"
          />
          <h3>{{ editor.login }}</h3>
          <OutlinedButton @click="removeEditor(editor.login)" :disabled="loading" class="ml-auto"
            >Remove</OutlinedButton
          >
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

export default defineComponent({
  name: 'EditorsDashboard',
  components: { TextField, OutlinedButton },

  setup() {
    const api = useApi();

    const { error, loading } = asyncRefs();
    const editors = ref<TwitchUser[]>([]);
    tryAsync(async() => {
      editors.value = await api.getEditors();
    }, loading, error);

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
