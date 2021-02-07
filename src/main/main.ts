import { app, BrowserWindow } from "electron";
import path from "path";
import { hello } from "../native";

let mainWindow: BrowserWindow | undefined;
function createWindow() {
  mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    title: "Wooting Analog Midi",
    webPreferences: {
      nodeIntegration: true,
    },
  });

  console.log("Creating window");
  mainWindow.loadFile(path.join(__dirname, "../public/index.html"));

  // mainWindow.webContents.openDevTools();
}
// console.log(hello());
app.whenReady().then(createWindow);

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (!mainWindow) {
    createWindow();
  }
});
