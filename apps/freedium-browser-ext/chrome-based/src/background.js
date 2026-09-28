importScripts("rewrite.js");

const MENU_ID = "open-on-freedium";

async function mirrorOrigin() {
  const stored = await chrome.storage.local.get({
    mirror: Freedium.DEFAULT_MIRROR,
  });
  try {
    return Freedium.normalizeMirror(stored.mirror);
  } catch {
    return Freedium.DEFAULT_MIRROR;
  }
}

async function flashUnsupported(tabId) {
  if (tabId == null) return;
  await chrome.action.setBadgeBackgroundColor({ color: "#8A3A2A", tabId });
  await chrome.action.setBadgeText({ text: "—", tabId });
  setTimeout(() => {
    chrome.action.setBadgeText({ text: "", tabId });
  }, 1600);
}

async function openOnFreedium(pageUrl, tab, newTab) {
  const target = Freedium.toFreediumUrl(pageUrl, await mirrorOrigin());
  if (!target) {
    await flashUnsupported(tab?.id);
    return;
  }
  if (newTab) {
    const created = { url: target, active: true };
    if (tab?.id != null) created.openerTabId = tab.id;
    if (tab && Number.isInteger(tab.index)) created.index = tab.index + 1;
    await chrome.tabs.create(created);
    return;
  }
  if (tab?.id != null) {
    await chrome.tabs.update(tab.id, { url: target });
    return;
  }
  await chrome.tabs.create({ url: target, active: true });
}

async function activeTab() {
  const [tab] = await chrome.tabs.query({
    active: true,
    currentWindow: true,
  });
  return tab;
}

chrome.action.onClicked.addListener((tab) => {
  openOnFreedium(tab.url, tab, false);
});

chrome.commands.onCommand.addListener(async (command) => {
  if (command !== "open-on-freedium") return;
  const tab = await activeTab();
  if (!tab) return;
  await openOnFreedium(tab.url, tab, false);
});

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: MENU_ID,
      title: "Open on Freedium",
      contexts: ["page", "link", "action"],
    });
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  const pageUrl = info.linkUrl || info.pageUrl || tab?.url;
  const newTab = Boolean(info.linkUrl);
  openOnFreedium(pageUrl, tab, newTab);
});
