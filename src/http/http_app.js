
function open_config() {
  console.log('opening config...');
  alert("Configuration currently unimplemented.");
}

function app_main() {
  window.main_elm = document.getElementById('main');
  window.simple_search_elm = document.getElementById('simple_search');
  window.simple_search_elm.addEventListener('keyup', function(e) {
    // Send full text to server
    window.app_ws.send(window.simple_search_elm.value);
  });
  window.websocket_retries_left = 10;
  setup_ws();
}

function setup_ws() {
  if (window.websocket_retries_left < 0) {
    // Reload page
    location.reload();
    return;
  }
  window.websocket_retries_left -= 1;
  
  window.app_websocket_url = 'ws://'+location.hostname+':'+window.client_http_websocket_port;
  try {
    window.app_ws = new WebSocket(window.app_websocket_url);
  }
  catch (e) {
    //window.simple_search_elm.disabled = true;
    setTimeout(setup_ws, 250);
    throw e;
  }
  
  window.app_ws.addEventListener('open', function (event) {
    window.simple_search_elm.disabled = false;
  });
  window.app_ws.addEventListener('close', function (event) {
    //window.simple_search_elm.disabled = true;
    setTimeout(setup_ws, 250);
  });
  window.app_ws.addEventListener('message', function (event) {
    var o = JSON.parse(event.data);
    // Check if server wants us to clear results first
    var action = o['action'];
    if (action && action == 'clear') {
      // Remove all results
      while (window.main_elm.hasChildNodes()) {
        window.main_elm.removeChild(window.main_elm.lastChild);
      }
    }
    // Create new results from all given {'records':[]}
    var records = o['records'];
    for (var i=0; i<records.length; i++) {
      add_record(records[i]["p"]); // just pass in map of key:value pairs
    }
  });
}

function add_record(record) {
  var elm = document.createElement('div');
  elm.classList.add('result');
  var rendered = false;
  try {
    for (var i=0; i<window.custom_renderers.length; i++) {
      if (window.custom_renderers[i]['can_render_func'](record)) {
        elm.appendChild(
          window.custom_renderers[i]['render_func'](record)
        );
        rendered = true;
        break;
      }
    }
  }
  catch (e) {
    console.log(e);
  }
  // Render as <pre> with JSON contents
  
  if (!rendered) {
    var p = document.createElement('pre');
    p.innerText = JSON.stringify(record);
    elm.appendChild(p);
  }
  
  window.main_elm.appendChild(elm);
}

// Add API for custom JS to add rendering logic
window.custom_renderers = [];
window.add_renderer = function(can_render_func, render_func) {
  window.custom_renderers.push({
    'can_render_func': can_render_func,
    'render_func': render_func
  });
};
