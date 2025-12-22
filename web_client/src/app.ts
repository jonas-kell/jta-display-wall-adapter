import { createApp } from "vue";
import "./styles/styles.scss";
import App from "./App.vue";
import router from "./router.ts";
import { createPinia } from "pinia";
import vuetify from "./plugins/vuetify.ts";
const pinia = createPinia();

createApp(App).use(vuetify).use(router).use(pinia).mount("#app");
