import { createApp } from 'vue'
import App from './App.vue'
import './index.css'
import router from "./router";
import ApiPlugin from './api/plugin';
import { DataStorePlugin } from './store';

createApp(App).use(router).use(ApiPlugin).use(DataStorePlugin).mount('#app')
