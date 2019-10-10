/**
 * Users can set the client_http_custom_js key in their .dindex.toml file
 * to replace content here.
 */

window.add_renderer(function(record) {
  // Can this renderer handle <record>?
  return ('title' in record) && ('url' in record);
},
function(record) {
  // Must return elm, renders the record to contents of a <div class="result">
  var elm = document.createElement('div');
  
  var title = document.createElement('h3');
  var url = document.createElement('a');
  
  title.innerText = record['title'];
  url.href = record['url'];
  url.innerText = record['url'];
  
  elm.appendChild(title);
  elm.appendChild(url);
  
  return elm;
});
