import { createWebHashHistory, createRouter } from "vue-router";

import Index from "./components/Index.vue";
import Heats from "./components/Heats.vue";
import WindRequest from "./components/WindRequest.vue";
import PDFSettings from "./components/PDFSettings.vue";
import PDFPrint from "./components/PDFPrint.vue";
import Timing from "./components/Timing.vue";
import ManageAthletes from "./components/ManageAthletes.vue";
import Debug from "./components/Debug.vue";

const routes = [
    { path: "/", component: Index, name: "index" },
    { path: "/heats", component: Heats, name: "heats" },
    { path: "/timing", component: Timing, name: "timing" },
    { path: "/manage_athletes", component: ManageAthletes, name: "manage_athletes" },
    { path: "/wind_request", component: WindRequest, name: "wind_request" },
    { path: "/pdf_settings", component: PDFSettings, name: "pdf_settings" },
    { path: "/pdf_print", component: PDFPrint, name: "pdf_print" },
    { path: "/debug", component: Debug, name: "debug" },
    { path: "/:catchAll(.*)*", redirect: "/" },
];

export default createRouter({
    history: createWebHashHistory(),
    routes,
});
