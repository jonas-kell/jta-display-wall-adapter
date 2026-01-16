import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import useMainStore from "./main";
import router from "../router";

export default defineStore("auth", () => {
    const mainStore = useMainStore();

    const PASSWORD_KEY = "TOTALLY_SECURE_PASSWORD_STORAGE_KEY";
    const password = ref(localStorage.getItem(PASSWORD_KEY) ?? "");
    watch(password, () => {
        localStorage.setItem(PASSWORD_KEY, password.value);
    });

    const authenticated = computed(() => {
        return mainStore.managementPassword == password.value;
    });

    const passwordComparisonLoaded = computed(() => {
        return mainStore.managementPassword != null;
    });

    watch(
        () => mainStore.managementPassword,
        (now, prev) => {
            if (prev == null && now != null) {
                if (password.value != now) {
                    // trigger navigation (re-avaluates "can I be here" rule)
                    router.push({ name: router.currentRoute.value.name });
                }
            }
        }
    );

    return {
        authenticated,
        password,
        passwordComparisonLoaded,
    };
});
