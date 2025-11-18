import { defineStore } from "pinia";
import { ref } from "vue";

export default defineStore("main", () => {
    const connected = ref(false);

    return { connected };
});
