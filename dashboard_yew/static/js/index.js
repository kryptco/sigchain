// generic functions
function hasClass(element, clas){
  if (element.className.indexOf(clas) > -1){
    return true
  } else {
    return false
  }
}

function addClass(name, clas){
  var element = document.getElementById(name);
  element.classList.add(clas);
}

function removeClass(name, clas){
  var element = document.getElementById(name);
  element.classList.remove(clas);
}

function toggleClass(name, clas){
  var element = document.getElementById(name);
  element.classList.toggle(clas);
}



// loader elements
function loaderStates() {
  var request, approve, fail, success, bgd;
  request = document.getElementById('request-loader');
  approve = document.getElementById('approve-loader');
  fail = document.getElementById('fail-loader');
  success = document.getElementById('success-loader');
  bgd = document.getElementById('loader-bg');

  if (hasClass(request, 'show')){
    request.classList.remove('show');
    approve.classList.add('show');
  } else if (hasClass(approve, 'show')){
    approve.classList.remove('show');
    fail.classList.add('show');
  } else if (hasClass(fail, 'show')){
    fail.classList.remove('show');
    success.classList.add('show');
  } else if (hasClass(success, 'show')){
    success.classList.remove('show');
    request.classList.add('show');
  } else {
    request.classList.add('show');
    bgd.classList.add('show');
  }
}

function hideLoader() {
  var request, approve, fail, success, bgd;
  request = document.getElementById('request-loader');
  approve = document.getElementById('approve-loader');
  fail = document.getElementById('fail-loader');
  success = document.getElementById('success-loader');
  bgd = document.getElementById('loader-bg');

  request.classList.remove('show');
  approve.classList.remove('show');
  fail.classList.remove('show');
  success.classList.remove('show');
  bgd.classList.remove('show');
}

function startLoader() {
  var request, approve, fail, success, bgd;
  request = document.getElementById('request-loader');
  approve = document.getElementById('approve-loader');
  fail = document.getElementById('fail-loader');
  success = document.getElementById('success-loader');
  bgd = document.getElementById('loader-bg');

  request.classList.remove('show');
  approve.classList.remove('show');
  fail.classList.remove('show');
  success.classList.remove('show');
  bgd.classList.remove('show');

  request.classList.add('show');
  bgd.classList.add('show');
}

function finishLoaderSuccess() {
  var request, approve, fail, success, bgd;
  request = document.getElementById('request-loader');
  approve = document.getElementById('approve-loader');
  fail = document.getElementById('fail-loader');
  success = document.getElementById('success-loader');
  bgd = document.getElementById('loader-bg');

  request.classList.remove('show');
  approve.classList.remove('show');
  fail.classList.remove('show');
  success.classList.remove('show');
  bgd.classList.remove('show');

  success.classList.add('show');
  bgd.classList.add('show');
  setTimeout(hideLoader, 1000);
}



// sidebar functions
function closeSidebar() {
  var bar, overlay;
  bar = document.getElementById('sidebar');
  overlay = document.getElementById('sidebar-overlay');

  bar.classList.remove('open');
  overlay.classList.remove('open');
}

function openSidebar(){
  var bar, overlay;
  bar = document.getElementById('sidebar');
  overlay = document.getElementById('sidebar-overlay');

  bar.classList.add('open');
  overlay.classList.add('open');
}

function showSideTab(opentab){
  var sidebar, tabs, open, menu, tab;
  sidebar = document.getElementById('sidebar');
  tabs = sidebar.getElementsByClassName('tab');
  open = document.getElementById(opentab);
  menu = sidebar.getElementsByTagName('li');
  tab = document.getElementById(opentab + '-title');

  for (var i = 0; i < tabs.length; i++){
    tabs[i].classList.remove('show');
  }
  for (var j = 0; j < menu.length; j++){
    menu[j].classList.remove('active');
  }
  open.classList.add('show');
  tab.classList.add('active');
}


// testing loading splash
function loadSplash(){
  var page, splash, progress;
  page = document.getElementById('page');
  splash = document.getElementById('loading-page');
  progress = document.getElementById('progress');

  page.classList.toggle('blurry');
  splash.classList.toggle('load');
  progress.classList.toggle('done'); // in reality you would update the progress transformY
}

function startSplashLoading() {
    progress = document.getElementById('progress');
    progress.classList.add('done');
}

function hideSplash() {
    setTimeout(function() {
        splash = document.getElementById('loading-page');
        splash.classList.remove('load');
    }, 100);
}



// testing loading bar
function showLoading(){
  var bar = document.getElementById('loading-bar');
  var load = document.getElementById('loaded');

  bar.style.visibility = 'visible';
  load.style.visibility = 'visible';
  bar.style.animationName = 'fadeIn';
  bar.style.animationDuration = '1s';
  load.style.animationName = 'fadeIn';
  load.style.animationDuration = '1s';

  setTimeout(function() {updateLoading(bar, load)}, 800);
}

var i = 0;

function updateLoading(bar, load){ //should maybe have a timeout
  if (i < 101){
    load.style.width = (i).toString() + '%';
    i += 10;
    setTimeout(updateLoading, 85, bar, load);
  } else {
    setTimeout(function() {closeLoading(bar, load)}, 500);
  }
}

function closeLoading(bar, load){
  bar.style.animationName = 'fadeOut';
  bar.style.animationDuration = '1s';
  load.style.animationName = 'fadeOut';
  load.style.animationDuration = '1s';

  setTimeout(function() {resetLoading(bar, load)}, 1000);
}

function resetLoading(bar, load){
  bar.style.visibility = 'hidden';
  load.style.visibility = 'hidden';
  bar.style.animationName = undefined;
  load.style.animationName = undefined;
  i = 0;
  load.style.width = 0;
  console.log('done')
}

function loadAnimate(){ //showy version
  var bar = document.getElementById('loading-bar');
  var load = document.getElementById('loaded');

  bar.style.animationName = 'loading';
  bar.style.animationDuration = '5s';
  load.style.animationName = 'loaded';
  load.style.animationDuration = '5s';
}



// drawing charts
function drawChart(keys, values, analytic){
  var chart = document.getElementById(analytic + '-chart');
  var line = document.getElementById(analytic + '-line');
  var clip = document.getElementById(analytic + '-clipoly');
  var axis = document.getElementById(analytic + '-axis');
  var nums = values;
  var num_map = [10, 80, 160, 230];
  var max = Math.max.apply(null, nums);
  var months = keys;
  var month_map = [0, 70, 140, 210];

  // clear points
  line.setAttribute('points', '');
  clip.setAttribute('points', '10,110');
  while(axis.lastChild){
    axis.removeChild(axis.lastChild);
  }

  for (var m in months) {
    var text = document.createElementNS("http://www.w3.org/2000/svg", "text");
    text.setAttributeNS(null, "x", month_map[m]);
    text.setAttributeNS(null, "y", 120);
    text.appendChild(document.createTextNode(months[m]));
    axis.appendChild(text);
  }

  for (var n in nums) {
    var point = chart.createSVGPoint();
    point.x = num_map[n];
    point.y = 105 - (100 / max) * nums[n];
    line.points.appendItem(point);
    clip.points.appendItem(point);
  }
  var endclip = chart.createSVGPoint();
  endclip.x = 230;
  endclip.y = 110;
  clip.points.appendItem(endclip);

  chart.style.animationName = 'loadChart';
  chart.style.animationDuration = '1s';
  setTimeout(function(){chart.style.animationName = ''}, 1200);
}

function initCharts(months, employees, sshs, signs){
  drawChart(months, employees, 'employee');
  drawChart(months, sshs, 'ssh');
  drawChart(months, signs, 'signs');

  setTimeout(function(){
      drawChart(months, [0, 2, 2, 6], 'employee');
    }, 1500) // example update of a chart
}



// settings inputs
function editTeam(){
  var input = document.getElementById('team-input');
  var button = document.getElementById('team-edit');

  if (input.className.indexOf('disabled') > -1){
    input.readOnly = false;
    button.textContent = 'Save';
    input.classList.toggle('disabled');
  } else {
    input.readOnly = true;
    button.textContent = 'Edit';
    input.classList.toggle('disabled');
  }
}

function editWindow() {
  var status_label = document.getElementById('status-label');
  var hour_text = document.getElementById('hr-text');
  var hour = document.getElementById('window-hr-input');
  var hour_label = document.getElementById('hour-label');
  var min_text = document.getElementById('min-text');
  var min = document.getElementById('window-min-input');
  var min_label = document.getElementById('min-label');
  var button = document.getElementById('window-edit');

  if (hour.className.indexOf('disabled') > -1) {
    if (status_label.style.display === "inline" && status_label.textContent === "NOT SET") {
      hour.value = '3';
    }
    status_label.style.display = "none";
    button.textContent = 'Save';
    min_text.style.display = "none";
    hour_text.style.display = "none";
    min_label.style.display = "inline";
    hour_label.style.display = "inline";
    min.classList.toggle('disabled');
    hour.classList.toggle('disabled');
  } else {
    status_label.style.display = "none";
    if ((min.value === '0' || min.value === null) && (hour.value === '0' || hour.value === null)) {
      status_label.style.display = "inline";
      status_label.textContent = "DISABLED";
    } else {
      status_label.style.display = "none";
      status_label.textContent = "";
    }
    if (hour.value === '0' || hour.value === null) {
      hour_label.style.display = "none";
    } else {
      hour_text.style.display = "inline";
    }
    if (min.value === '0' || min.value === null) {
      min_label.style.display = "none";
    } else {
      min_text.style.display = "inline";
    }
    button.textContent = 'Edit';
    hour.classList.toggle('disabled');
    min.classList.toggle('disabled');
  }
}

function unsetWindow() {
  var status_label = document.getElementById('status-label');
  status_label.style.display = "inline";
  status_label.textContent = "NOT SET";

  var hour_text = document.getElementById('hr-text');
  var hour = document.getElementById('window-hr-input');
  var hour_label = document.getElementById('hour-label');
  var min_text = document.getElementById('min-text');
  var min = document.getElementById('window-min-input');
  var min_label = document.getElementById('min-label');
  min_text.style.display = "none";
  min_label.style.display = "none";
  hour_text.style.display = "none";
  hour_label.style.display = "none";
  min.classList.add('disabled');
  hour.classList.add('disabled');

  var button = document.getElementById('window-edit');
  button.textContent = 'Edit';
}


// add new member
function openAddTeam(){
  addClass('team-bg', 'show');
  addClass('team-container', 'show');
  addClass('options', 'show');
}

function closeAddTeam() {
  var box = document.getElementById('option-details');
  var list = document.getElementById('emails');
  var bg = document.getElementById('team-bg');
  var all = ['team-bg', 'team-container', 'options', 'ind-title', 'team-title', 'enter-ind', 'add-link', 'option-details', 'link-load', 'mail-link', 'ind-create'];

  if (bg.className.indexOf('unclosable') <= -1){
    for (var a in all){
      removeClass(all[a], 'show');
    }
    box.style.height = 0;
    removeClass('option-details', 'sandwich');
    while(list.lastChild){
      list.removeChild(list.lastChild);
    }
  }
}

function allowAddTeamClose() {
  removeClass('team-bg', 'unclosable');
}

function addCloseIcon() {
  var closeIcon = document.getElementById('close-icon');
  closeIcon.style.visibility = 'visible';
}

function addInd(){
  var list = document.getElementById('emails');
  var input = document.getElementById('ind-email');
  var item = document.createElement('LI');
  if ((input != '' | input != null) & input.value.indexOf('@') > -1){
    // add to list
    item.textContent = input.value.replace(/\s/g, '');;
    list.appendChild(item);

    // increase height
    var box = document.getElementById('option-details');
    var height = box.style.height;
    var int_height = parseInt(height.substring(0, height.indexOf('p')))
    box.style.height = (int_height + 25).toString() + 'px';
    input.value = '';

    createMailto();
  }
}

function getEmails() {
    var el = document.getElementById("emails");
    var emailsArray = new Array();

    for (i = 0; i < el.children.length; i++) {
        emailsArray[i] = el.children[i].textContent;
    }

    return emailsArray;
}

function openIndLink(){
  var box = document.getElementById('option-details');
  box.style.height = '100px';
  removeClass('options', 'show');
  addClass('ind-title', 'show');
  addClass('enter-ind', 'show');
  addClass('option-details', 'show');
  addClass('mail-link', 'show');
  addClass('ind-create', 'show');
  addClass('option-details', 'sandwich');
}

function openTeamLink(){
  var box = document.getElementById('option-details');
  removeClass('options', 'show');
  addClass('team-title', 'show');
  addClass('option-details', 'show');
}

function createLink(){
  var box = document.getElementById('option-details');
  var status = document.getElementById('link-load-text');
  status.textContent = "Requesting approval from Krypton";
  var closeIcon = document.getElementById('close-icon');
  closeIcon.style.visibility = 'hidden';
  addClass('link-load', 'show');
  allowAddTeamClose();
  removeClass('enter-ind', 'show');
  setTimeout(function() {status.textContent = "Phone approval required"}, 1000);
  box.style.height = '45px';
}

function showLink(){
  var box = document.getElementById('option-details');
  var closeIcon = document.getElementById('close-icon');
  closeIcon.style.visibility = 'visible';
  allowAddTeamClose();
  removeClass('link-load', 'show');
  addClass('add-link', 'show');
  box.style.height = '65px';
}

function createIndLink(){
  var emails = document.getElementById('emails');
  var input = document.getElementById('ind-email');
  if (emails.lastChild){
    removeClass('ind-create', 'show');
    removeClass('option-details', 'sandwich');
    createMailto();
    createLink();
  } else {
    input.style.animationName = 'shake';
    input.style.animationDuration = '.6s';
    addClass('ind-create', 'oops');
    setTimeout(function(){input.style.animationName = ''; removeClass('ind-create', 'oops')}, 1000)
  }
}

function createMailto(){
  var emails = document.getElementById('emails').children;
  var list = [];
  var button = document.getElementById('mail-link');
  var self = `self@krypt.co`;
  var mailto = `mailto:`;
  for (var i = 0; i < emails.length; i++){
    list.push(emails[i].textContent);
  }
  mailto += list.join();
  mailto += `?cc=self@krypt.co` + `&subject=Invitation%20to%20Krypton%20Teams%21&body=You%27re%20invited%20to%20join%20Team%20Awesome%20on%20Krypton%20Teams%21%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20Step%201.%20Download%3A%20%20https%3A//get.krypt.co%0A%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20%20Step%202.%20Visit%20the%20link%20below%20on%20your%20phone`;
  button.href = mailto;
}

//  https://stackoverflow.com/questions/400212/how-do-i-copy-to-the-clipboard-in-javascript
function copyTextToClipboard(text) {
  var textArea = document.createElement("textarea");

  //
  // *** This styling is an extra step which is likely not required. ***
  //
  // Why is it here? To ensure:
  // 1. the element is able to have focus and selection.
  // 2. if element was to flash render it has minimal visual impact.
  // 3. less flakyness with selection and copying which **might** occur if
  //    the textarea element is not visible.
  //
  // The likelihood is the element won't even render, not even a flash,
  // so some of these are just precautions. However in IE the element
  // is visible whilst the popup box asking the user for permission for
  // the web page to copy to the clipboard.
  //

  // Place in top-left corner of screen regardless of scroll position.
  textArea.style.position = 'fixed';
  textArea.style.top = 0;
  textArea.style.left = 0;

  // Ensure it has a small width and height. Setting to 1px / 1em
  // doesn't work as this gives a negative w/h on some browsers.
  textArea.style.width = '2em';
  textArea.style.height = '2em';

  // We don't need padding, reducing the size if it does flash render.
  textArea.style.padding = 0;

  // Clean up any borders.
  textArea.style.border = 'none';
  textArea.style.outline = 'none';
  textArea.style.boxShadow = 'none';

  // Avoid flash of white box if rendered for any reason.
  textArea.style.background = 'transparent';


  textArea.value = text;

  document.body.appendChild(textArea);

  textArea.select();

  try {
    var successful = document.execCommand('copy');
    var msg = successful ? 'successful' : 'unsuccessful';
  } catch (err) {
    console.log('Oops, unable to copy');
  }

  document.body.removeChild(textArea);
}
