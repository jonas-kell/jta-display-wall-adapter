import { createWebHashHistory, createRouter } from "vue-router";

import Index from "./components/Index.vue";
import WindRequest from "./components/WindRequest.vue";
import PDFTest from "./components/PDFTest.vue";

const routes = [
    { path: "/", component: Index, name: "index" },
    { path: "/wind_request", component: WindRequest, name: "wind_request" },
    { path: "/pdf_test", component: PDFTest, name: "pdf_test" },
    { path: "/:catchAll(.*)*", component: Index },
];

export default createRouter({
    history: createWebHashHistory(),
    routes,
});
