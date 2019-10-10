
function open_config() {
  console.log("opening config...");
}

function app_main() {
  window.main_elm = document.getElementById('main');
  window.simple_search_elm = document.getElementById('simple_search');
  window.simple_search_elm.addEventListener('keyup', function(e) {
    // Send full text to server
    window.app_ws.send(window.simple_search_elm.value);
  });
  setup_ws();
}

function setup_ws() {
  window.app_websocket_url = 'ws://'+location.hostname+':'+window.client_http_websocket_port;
  window.app_ws = new WebSocket(window.app_websocket_url);
  window.app_ws.addEventListener('open', function (event) {
    window.simple_search_elm.disabled = false;
  });
  window.app_ws.addEventListener('close', function (event) {
    window.simple_search_elm.disabled = true;
    setTimeout(setup_ws, 600);
  });
  window.app_ws.addEventListener('message', function (event) {
    var o = JSON.parse(event.data);
    // Remove all results
    while (window.main_elm.hasChildNodes()) {
      window.main_elm.removeChild(window.main_elm.lastChild);
    }
    // Create new results
    var records = o["records"];
    for (var i=0; i<records.length; i++) {
      add_record(records[i]);
    }
  });
}

function add_record(record) {
  var elm = document.createElement('div');
  elm.classList.add('result');
  
  elm.innerText = record;
  
  window.main_elm.appendChild(elm);
}
