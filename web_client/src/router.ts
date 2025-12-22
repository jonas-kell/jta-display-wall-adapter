import { createWebHashHistory, createRouter } from "vue-router";

import Index from "./components/Index.vue";
import WindRequest from "./components/WindRequest.vue";
import PDFTest from "./components/PDFTest.vue";
import Timing from "./components/Timing.vue";
import ManageAthletes from "./components/ManageAthletes.vue";

const routes = [
    { path: "/", component: Index, name: "index" },
    { path: "/timing", component: Timing, name: "timing" },
    { path: "/manage_athletes", component: ManageAthletes, name: "manage_athletes" },
    { path: "/wind_request", component: WindRequest, name: "wind_request" },
    { path: "/pdf_test", component: PDFTest, name: "pdf_test" },
    { path: "/:catchAll(.*)*", redirect: "/" },
];

export default createRouter({
    history: createWebHashHistory(),
    routes,
});
