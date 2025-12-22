// Vuetify
import { createVuetify } from "vuetify";

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
import * as components from "vuetify/components";
import * as directives from "vuetify/directives";
import { aliases, mdi } from "vuetify/iconsets/mdi";

// comment in and edit to change
const lightTheme = {
    dark: false,
    colors: {
        // background: "#FFFFF",
        "on-background": "#000000",
        // surface: "#FFFFFF",
        // "surface-bright": "#GGGGGG",
        // "surface-light": "#EEEEEE",
        // "surface-variant": "#424242",
        // "on-surface-variant": "#EEEEEE",
        // primary: "#1867C0",
        // "primary-darken-1": "#1F5592",
        // secondary: "#48A9A6",
        // "secondary-darken-1": "#018786",
        // error: "#B00020",
        // info: "#2196F3",
        // success: "#4CAF50",
        // warning: "#FB8C00",
        "alternative-row-color": "#E0E0E0",
        "fg-red": "#CC0000",
        "fg-green": "#007000",
        "fg-yellow": "#999100",
        "bg-red": "#CC0000",
        "bg-green": "#007000",
        "bg-yellow": "#FFE711",
        "statistics-male": "#1867C0",
        "statistics-female": "#C01867",
        "statistics-others": "#67C018",
        "statistics-grid-color": "#767676",
    },
    variables: {
        // "border-color": "#000000",
        // "border-opacity": 0.12,
        // "high-emphasis-opacity": 0.87,
        // "medium-emphasis-opacity": 0.6,
        // "disabled-opacity": 0.38,
        // "idle-opacity": 0.04,
        // "hover-opacity": 0.04,
        // "focus-opacity": 0.12,
        // "selected-opacity": 0.08,
        // "activated-opacity": 0.12,
        // "pressed-opacity": 0.12,
        // "dragged-opacity": 0.08,
        // "theme-kbd": "#212529",
        // "theme-on-kbd": "#FFFFFF",
        // "theme-code": "#F5F5F5",
        // "theme-on-code": "#000000",
        "enable-alternative-row-color": 1, // to disable, set to 0, to enable to 1
    },
};

const darkTheme = {
    dark: true,
    colors: {
        "on-background": "#FFFFFF",
        "alternative-row-color": "#1A1A1A",
        "fg-red": "#FF4444",
        "fg-green": "#00FF00",
        "fg-yellow": "#FFFF00",
        "bg-red": "#FF4444",
        "bg-green": "#00FF00",
        "bg-yellow": "#FFFF00",
        "statistics-male": "#1867C0",
        "statistics-female": "#C01867",
        "statistics-others": "#67C018",
        "statistics-grid-color": "#858585",
    },
    variables: {
        "enable-alternative-row-color": 1, // to disable, set to 0, to enable to 1
    },
};

export default createVuetify({
    components,
    directives,
    theme: {
        defaultTheme: "dark",
        themes: {
            light: lightTheme,
            dark: darkTheme,
        },
    },
    icons: {
        defaultSet: "mdi",
        aliases,
        sets: {
            mdi,
        },
    },
});
