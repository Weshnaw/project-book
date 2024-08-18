import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { debug } from "@tauri-apps/plugin-log";

import htmx from "htmx.org";

/* HTMX plugin */
const COMMAND_PREFIX = "command:";

const patchedSend = async function (this: any, params: any) {
  // Make readonly properties writable
  Object.defineProperty(this, "readyState", { writable: true });
  Object.defineProperty(this, "status", { writable: true });
  Object.defineProperty(this, "statusText", { writable: true });
  Object.defineProperty(this, "response", { writable: true });

  // Set response
  const query = new URLSearchParams(params);
  debug(`HTMX tauri invoke ${this.command} with ${JSON.stringify(query)}`);
  this.response = await invoke(this.command, Object.fromEntries(query));
  this.readyState = XMLHttpRequest.DONE;
  this.status = 200;
  this.statusText = "OK";

  // We only need load event to trigger a XHR response
  this.dispatchEvent(new ProgressEvent("load"));
};

window.addEventListener("DOMContentLoaded", () => {
  debug(`DOMContentLoaded`);
  document.body.addEventListener("htmx:beforeSend", (event: any) => {
    const path = event.detail.requestConfig.path;
    if (path.startsWith(COMMAND_PREFIX)) {
      event.detail.xhr.command = path.slice(COMMAND_PREFIX.length);
      event.detail.xhr.send = patchedSend;
    }
  });
});
/* END HTMX pluigin */

listen("update-settings", (_) => {
  debug(`update-settings event`);
  htmx.trigger(htmx.find("body")!, "update-settings", null);
});

(<any>window).updateServer = async () => {
  debug("updateServer triggered");
  let input: HTMLInputElement = document.querySelector("#server-input")!;
  await invoke("plex_update_server", {
    server: input.value,
  });
};
(<any>window).updateLibrary = async () => {
  debug("updateLibrary triggered");
  let input: HTMLInputElement = document.querySelector("#library-input")!;
  await invoke("plex_update_library", {
    library: input.value,
  });
};

class BookCard extends HTMLElement {
  thumb: string = "";
  title: string = "";
  author: string = "";

  constructor() {
    super();
    this.attachShadow({ mode: "open" });

    this.render();
  }

  // Watch for attribute changes
  static get observedAttributes() {
    return ["thumb", "title", "author"];
  }

  // Sync attribute changes with properties
  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ) {
    debug(`book-card ${name}, ${oldValue}, ${newValue}`);
    if (name === "thumb") {
      this.thumb = newValue!;
    }
    if (name === "title") {
      this.title = newValue!;
    }
    if (name === "author") {
      this.author = newValue!;
    }

    this.render();
  }

  // <img src="${this.thumb}" alt="${this.title}">
  // <span>${this.title}</span><br />
  // <span>${this.author}</span><br />
  render() {
    this.shadowRoot!.innerHTML = `
      <style>
        .card {
          transition: 0.3s;
          border-radius: 5px; /* 5px rounded corners */
          height: 100%
        }
        img {
          border-radius: 5px 5px 0 0;
          width: 100%;
          height: 100%;
        }
        .card:hover {
          box-shadow: 0 8px 16px 0 rgba(0,0,0,0.2);
        }
        .text {
          padding: 2px 16px;
          overflow: hidden;
          white-space: nowrap; /* Don't forget this one */
          text-overflow: ellipsis;
        }
        .title {
          padding: 2px 16px;
        }
      </style>
      <div class="card">
        <img src="${this.thumb}" alt="${this.title}">
        <div class="text">
          ${this.title}<br />
          <sub>${this.author}</sub>
        </div>
      </div>
    `;
  }
}

customElements.define("book-card", BookCard);
