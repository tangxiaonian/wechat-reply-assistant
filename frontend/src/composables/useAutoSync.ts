import { onMounted, onUnmounted, watch } from "vue";
import { useWorkspace } from "../stores/workspace";
import { syncSettings, windowVisible, errorMessage } from "../bridge";

/** One timer for the selected conversation; hidden windows and active generation never poll. */
export function useAutoSync(): void {
  const store = useWorkspace();
  let timer: ReturnType<typeof setInterval> | undefined;
  function schedule(): void {
    clearInterval(timer);
    if (!store.syncPreferences.enabled) {
      return;
    }
    timer = setInterval(() => {
      void tick();
    }, store.syncPreferences.intervalSeconds * 1000);
  }
  async function tick(): Promise<void> {
    if (
      store.generating ||
      store.refreshingContext ||
      store.manualTranscript ||
      !store.contact
    ) {
      return;
    }
    try {
      if (await windowVisible()) {
        await store.refreshContext(true);
      }
    } catch (error) {
      store.error = errorMessage(error);
    }
  }
  watch(
    () => [
      store.syncPreferences.enabled,
      store.syncPreferences.intervalSeconds,
      store.selectedId,
    ],
    schedule,
  );
  onMounted(async () => {
    try {
      store.syncPreferences = await syncSettings();
    } catch (error) {
      store.error = errorMessage(error);
    }
  });
  onUnmounted(() => {
    clearInterval(timer);
  });
}
