var active_id = undefined;
var port = "58810"; // TODO make this configuraable

const timestamp = () => Math.round((new Date()).getTime() / 1000);

// const set_port = user_port => { port = user_port; }

const set_inactive = id => {
    console.log(`setting inactive on tab: ${active_id}`);
    active_id = undefined;
    browser.browserAction.setIcon({ path: "icons/icon.svg", tabId: id });
    browser.tabs.onUpdated.removeListener(listener);
}

const set_active = id => {
    active_id = id;
    console.log(`setting active on tab: ${active_id}`);
    browser.browserAction.setIcon({ path: "icons/icon-active.svg", tabId: id });
    const filter = { tabId: id }
    browser.tabs.onUpdated.addListener(listener, filter);
}

async function log_url(url) {
    let ts = timestamp();
    console.log(`sending '${url}' @ '${ts}'`);
    try {
        let body = JSON.stringify({
            id: url,
            ts: ts,
        });
        let resp = await fetch(`http://localhost:${port}/youtube`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: body
        });
        console.log(resp);
    } catch (ex) {
        console.error("error sending request: {}", ex);
    }
};

async function listener(id, info, tab) {
    console.log("listener was called");
    if (info.url) {
        browser.browserAction.setIcon({ path: "icons/icon-active.svg", tabId: id });
        await log_url(tab.url);
    }
    return true;
}

browser.browserAction.onClicked.addListener(tab => {
    console.log("button was clicked");

    if (active_id === tab.id) {
        console.log("setting inactive:", active_id);
        set_inactive(active_id);
        return;
    }

    if (active_id) {
        console.log("setting inactive:", active_id);
        set_inactive(active_id);
    }
    console.log("setting active:", tab.id);
    set_active(tab.id)
});
