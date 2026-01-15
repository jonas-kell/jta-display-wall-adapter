import { defineStore } from "pinia";
import { ref } from "vue";

export default defineStore("debug", () => {
    const startTime = ref(null as null | number);
    const finishIndex = ref(-1);
    const times = ref([] as number[]);

    return {
        startTime,
        finishIndex,
        times,
    };
});
