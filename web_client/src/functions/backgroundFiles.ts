import { ref, watch } from "vue";
import { deleteString, loadString, storeString } from "./fileStorage";

export function backgroundFileManagement() {
    const IMAGE_KEY_BIB = "IMAGE_KEY_BIB";
    const IMAGE_KEY_BIB_LANDSCAPE = "IMAGE_KEY_BIB_LANDSCAPE";
    const IMAGE_KEY_CERTIFICATE = "IMAGE_KEY_CERTIFICATE";
    const IMAGE_KEY_CERTIFICATE_LANDSCAPE = "IMAGE_KEY_CERTIFICATE_LANDSCAPE";

    const processedBackgroundImageBibLandscape = ref((localStorage.getItem(IMAGE_KEY_BIB_LANDSCAPE) ?? "false") == "true");
    const processedBackgroundImageCertificateLandscape = ref(
        (localStorage.getItem(IMAGE_KEY_CERTIFICATE_LANDSCAPE) ?? "false") == "true"
    );
    watch(processedBackgroundImageBibLandscape, () => {
        localStorage.setItem(IMAGE_KEY_BIB_LANDSCAPE, String(processedBackgroundImageBibLandscape.value));
    });
    watch(processedBackgroundImageCertificateLandscape, () => {
        localStorage.setItem(IMAGE_KEY_CERTIFICATE_LANDSCAPE, String(processedBackgroundImageCertificateLandscape.value));
    });

    const processedBackgroundImageBib = ref(null as null | string);
    watch(processedBackgroundImageBib, () => {
        if (loaded) {
            if (processedBackgroundImageBib.value) {
                storeString(IMAGE_KEY_BIB, processedBackgroundImageBib.value);
            }
        }
    });
    const processedBackgroundImageCertificate = ref(null as null | string);
    watch(processedBackgroundImageCertificate, () => {
        if (loaded) {
            if (processedBackgroundImageCertificate.value) {
                storeString(IMAGE_KEY_CERTIFICATE, processedBackgroundImageCertificate.value);
            }
        }
    });

    function updateStoredBibBackground(data: { background: string; BGlandscape: boolean }) {
        processedBackgroundImageBibLandscape.value = data.BGlandscape;
        processedBackgroundImageBib.value = data.background;
    }
    function updateStoredCertificateBackground(data: { background: string; BGlandscape: boolean }) {
        processedBackgroundImageCertificateLandscape.value = data.BGlandscape;
        processedBackgroundImageCertificate.value = data.background;
    }
    function clearStoredBibBackground() {
        processedBackgroundImageBib.value = null;
        deleteString(IMAGE_KEY_BIB);
    }
    function clearStoredCertificateBackground() {
        processedBackgroundImageCertificate.value = null;
        deleteString(IMAGE_KEY_CERTIFICATE);
    }

    let loaded = false;
    async function initalLoad() {
        const dataBib = await loadString(IMAGE_KEY_BIB);
        if (dataBib) {
            processedBackgroundImageBib.value = dataBib;
        }
        const dataCertificate = await loadString(IMAGE_KEY_CERTIFICATE);
        if (dataCertificate) {
            processedBackgroundImageCertificate.value = dataCertificate;
        }

        loaded = true;
    }
    initalLoad();

    return {
        processedBackgroundImageBibLandscape,
        processedBackgroundImageCertificateLandscape,
        processedBackgroundImageBib,
        processedBackgroundImageCertificate,
        updateStoredBibBackground,
        updateStoredCertificateBackground,
        clearStoredBibBackground,
        clearStoredCertificateBackground,
    };
}
