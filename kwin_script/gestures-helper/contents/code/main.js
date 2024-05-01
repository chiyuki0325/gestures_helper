function notifyActiveWindow(client) {
  callDBus(
    "ink.chyk.GesturesHelper",
    "/ink/chyk/GesturesHelper",
    "ink.chyk.GesturesHelper",
    "NotifyActiveWindow",
    "caption" in client ? client.caption : "",
    "resourceClass" in client ? client.resourceClass : "",
    "resourceName" in client ? client.resourceName : "",
  );
}

workspace.windowActivated.connect(notifyActiveWindow);
