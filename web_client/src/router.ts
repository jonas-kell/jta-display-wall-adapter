import { createWebHashHistory, createRouter } from "vue-router";

import Index from "./components/Index.vue";
import Heats from "./components/Heats.vue";
import WindRequest from "./components/WindRequest.vue";
import PDFSettings from "./components/PDFSettings.vue";
import PDFPrint from "./components/PDFPrint.vue";
import Timing from "./components/Timing.vue";
import ManageAthletes from "./components/ManageAthletes.vue";
import Debug from "./components/Debug.vue";
import BibInterface from "./components/BibInterface.vue";

import useAuthStore from "./stores/auth";

const routes = [
    { path: "/", component: Index, name: "index", meta: { requiresNoAuth: true } },
    { path: "/heats", component: Heats, name: "heats" },
    { path: "/timing", component: Timing, name: "timing" },
    { path: "/manage_athletes", component: ManageAthletes, name: "manage_athletes" },
    { path: "/wind_request", component: WindRequest, name: "wind_request" },
    { path: "/pdf_settings", component: PDFSettings, name: "pdf_settings" },
    { path: "/pdf_print", component: PDFPrint, name: "pdf_print" },
    { path: "/debug", component: Debug, name: "debug" },
    { path: "/bib_interface", component: BibInterface, name: "bib_interface", meta: { requiresNoAuth: true } },
    { path: "/:catchAll(.*)*", redirect: "/", meta: { requiresNoAuth: true } },
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

router.beforeEach((to) => {
    const auth = useAuthStore();

    const requiresAuth = !(to.meta.requiresNoAuth === true);

    if (requiresAuth && !auth.authenticated) {
        if (auth.passwordComparisonLoaded) {
            // on page reload finish the navigation at first (we will navigate away, if password is loaded in the auth store)
            return {
                name: "index",
            };
        }
    }
});

export default router;
