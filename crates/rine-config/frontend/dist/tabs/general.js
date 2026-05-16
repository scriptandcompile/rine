// General tab: filesystem, Windows version, DLL, and environment configuration

function setupGeneralTab() {
  document.getElementById("add-drive").addEventListener("click", () => {
    addKvRow(document.getElementById("drive-list"), "", "", "A-Z", "Drive letter");
    markDirty();
  });

  document.getElementById("add-env").addEventListener("click", () => {
    addKvRow(document.getElementById("env-list"), "", "", "VARIABLE", "Name");
    markDirty();
  });

  document.getElementById("browse-root").addEventListener("click", async () => {
    const input = document.getElementById("default-root");
    const startDir = input.value.trim() || "~/.rine/drives";
    try {
      const folder = await window.__TAURI__.core.invoke("pick_folder", { startDir });
      if (folder) {
        input.value = folder;
        markDirty();
      }
    } catch (err) {
      // user cancelled or error — ignore
    }
  });
}
