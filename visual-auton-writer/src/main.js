const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
const { open, save, confirm } = window.__TAURI__.dialog;

var selected_mvmt = -1;

async function moveControlPoint(e, xPos, yPos) {
  const locations = await invoke("move_ctrl", {posX: xPos, posY: yPos});

  const [[[xVals, yVals],[ctrlXVals, ctrlYVals]], curr_pos] = locations;

  if (curr_pos[0] != 100 && curr_pos[1] != 100) {
    document.getElementById("curr-position").innerText = Math.round(curr_pos[0]*100)/100 + ", " + Math.round(curr_pos[1]*100)/100 // round to 2 decimals
  }

  addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
}

async function selectControlPoint(e, xPos, yPos) {
  if (e.button === 2) {
    const locations = await invoke("select_ctrl", {posX: xPos, posY: yPos});

    const [[xVals, yVals],[ctrlXVals, ctrlYVals]] = locations;

    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);

    document.getElementById("curr-position").style.display = "inline";
  }
}

async function deselectCtrl(e) {
  if (e.button === 2) {
    document.getElementById("curr-position").style.display = "none";
    await invoke("deselect_ctrl");
  }
}

function addPointsToHTML(xvals, yvals, ctrlXVals, ctrlYVals) {
  const container = document.getElementById('curve-container');
  container.innerHTML = ''; // Clear existing points

  const bot_render = document.createElement("span");
  bot_render.id = "bot-render"

  container.appendChild(bot_render);

  for (let i = 0; i < xvals.length; i++) {
    const point = document.createElement('div');
    point.className = 'point';
    point.style.left = `${xvals[i]-4}px`;
    point.style.top = `${yvals[i]-4}px`;

    container.appendChild(point);

    bot_render.style.left = `${xvals[i]-45}px`;
    bot_render.style.top = `${yvals[i]-45}px`;
  }

  for (let i = 0; i < ctrlXVals.length; i++) {
    const ctrl_point = document.createElement('div');
    ctrl_point.className = 'control-point';
    ctrl_point.style.left = `${ctrlXVals[i]-6}px`;
    ctrl_point.style.top = `${ctrlYVals[i]-6}px`;

    ctrl_point.addEventListener("mousedown", (e) => {
      e.preventDefault();
      selectControlPoint(e, ctrlXVals[i], ctrlYVals[i]);
    });

    // console.log(ctrlXVals[i], ctrlYVals[i]);

    container.appendChild(ctrl_point);
  }

  // console.log(container);
  // console.log(xvals, yvals);
}

function addMovementList(movementList) {
  const container = document.getElementById("movement-list");
  container.innerHTML = '';

  const endTolChange = document.getElementById("end-tolerance-change");
  const timeoutChange = document.getElementById("timeout-change");
  const specificChange = document.getElementById("specific-change");
  const currEdit = document.getElementById("curr-editing");

  for (let i = 0; i < movementList.length; i++) {
    const item = document.createElement('button');
    item.className = 'on-menu-movement-item';

    item.innerText = movementList[i].id.toString() + " - ???";

    if (movementList[i].lookahead != 0.0 ) {
      item.innerText = movementList[i].id.toString() + " - Follow";
    }
    else if (movementList[i].distance != 0) {
      item.innerText = movementList[i].id.toString() + " - Lateral";
    }
    else if (movementList[i].name.length > 0) {
      item.innerText = movementList[i].id.toString() + " - Command";
    }
    else {
      item.innerText = movementList[i].id.toString() + " - Turn";
    }

    item.addEventListener("mousedown", (e) => {
      console.log(movementList[i]);
      // send this item to config menu
      currEdit.innerText = "Editing: Movement w/ ID " + (movementList[i].id).toString(); 
      endTolChange.value = movementList[i].end_tol;
      timeoutChange.value = movementList[i].timeout;
      specificChange.value = ""
      if (movementList[i].distance != 0) {
        specificChange.value = movementList[i].distance;
      }
      else if (movementList[i].lookahead != 0) {
        specificChange.value = movementList[i].lookahead;
      }
      else if (movementList[i].name != "") {
        specificChange.value = movementList[i].name;
      }
      else {
        specificChange.value = movementList[i].angle * 180 / Math.PI;
      }

      selected_mvmt = movementList[i].id;

      console.log("selected mvmt ", selected_mvmt);
    })

    container.appendChild(item);
  }
}

async function detectClick(e) {
  if (e.button === 0) {
    const locations = await invoke("click", {posX: e.clientX, posY: e.clientY});

    const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;

    console.log(movementList);

    addMovementList(movementList);

    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
  }
}

async function saveFile() {
  try {
    // open the native file open dialog to choose a path
    const selectedPath = await save({
      title: 'Select Save Path',
      defaultPath: 'output.txt',
      filters: [
        {
          name: 'Text Files',
          extensions: ['txt', 'vaw'],
        },
      ],
      multiple: false,
      directory: false,
      save: true,
    });

    if (selectedPath) {
      console.log(`File saved at: ${selectedPath}`);
      return selectedPath
    } else {
      console.log('Cancelled file saving.');
    }
    return selectedPath;
  } catch (error) {
    console.error('Error choosing file path:', error);
    return "fail_to_save.txt";
  }
}

async function openFile() {
  try {
    // open the native file open dialog to choose a path
    const selectedPath = await open({
      title: 'Select Open Path',
      filters: [
        {
          name: 'Text Files',
          extensions: ['txt', 'vaw'],
        },
      ],
      multiple: false,
      directory: false,
      save: false,
    });

    if (selectedPath) {
      console.log(`File saved at: ${selectedPath}`);
      return selectedPath
    } else {
      console.log('Cancelled file saving.');
    }
    return selectedPath;
  } catch (error) {
    console.error('Error choosing file path:', error);
    return "fail_to_save.txt";
  }
}

async function exportPath(e) {
  var selectedPath = await saveFile()

  await invoke('export_file', {path: selectedPath});
}

async function loadPath(e) {
  var selectedPath = await openFile()

  console.log("Load file", selectedPath);

  const locations = await invoke('load_file', {path: selectedPath});
  const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;

  addMovementList(movementList);

  addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
}


async function clearPath(e) {

  const confirmed = await confirm('Are you sure you want to create a new route? This will wipe the current one.', 'Confirm');

  if (confirmed == true) {
    const locations = await invoke("clear");

    const [[xVals, yVals],[ctrlXVals, ctrlYVals], movementList] = locations;

    addMovementList(movementList);

    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
  }
}

async function changeStartPos(e) {
  var position = [
    parseFloat(document.getElementById("start-pos-x").value),
    parseFloat(document.getElementById("start-pos-y").value),
    parseFloat(document.getElementById("start-pos-heading").value)
  ]

  const locations = await invoke("change_start_pos", {position: position});

  const [[xVals, yVals],[ctrlXVals, ctrlYVals], movementList] = locations;

  addMovementList(movementList);

  addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
}

async function addRouteToControl(e) {
  var form_type = document.getElementById("add-item-type").value;
  var specific_value = document.getElementById("specific-val-input").value;

  if (form_type == 'command') {
    const locations = await invoke("add_command", {name: specific_value});
    const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;

    addMovementList(movementList);

    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
  }

  var end_tol_value = document.getElementById("end-tolerance-input").value;
  var timeout_value = document.getElementById("timeout-input").value;

  console.log(form_type, specific_value, end_tol_value, timeout_value );

  if (specific_value == "" || end_tol_value == "" || timeout_value =="") {
    return;
  }

  if (form_type == 'lateral') {
    const locations = await invoke("add_lateral_movement", {distance: parseFloat(specific_value), endtol: parseFloat(end_tol_value), timeout: parseFloat(timeout_value)});
    const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;

    addMovementList(movementList);

    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
  }
  else if (form_type == 'turn') {
    const locations = await invoke("add_turn_movement", {angle: parseFloat(specific_value), endtol: parseFloat(end_tol_value), timeout: parseFloat(timeout_value)});
    const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;

    addMovementList(movementList);

    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
  }
}

async function deleteLastMvmt(e) {
  const locations = await invoke("delete_last");
  const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;

  addMovementList(movementList);

  addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
}

async function updateSelectedRoute(e) {
  const endTolChange = document.getElementById("end-tolerance-change");
  const timeoutChange = document.getElementById("timeout-change");
  const specificChange = document.getElementById("specific-change");

  console.log("update selected route w/ ", endTolChange.value, timeoutChange.value, specificChange.value, selected_mvmt)

  if (endTolChange.value != "" && timeoutChange.value != "" && specificChange.value != "" && selected_mvmt != -1) {
    const locations = await invoke("change_mvmt",  {
      specific: specificChange.value,  endtol: parseFloat(endTolChange.value), timeout: parseFloat(timeoutChange.value), id: selected_mvmt
    });
    
    const [[xVals, yVals],[ctrlXVals, ctrlYVals],movementList] = locations;
  
    addMovementList(movementList);
  
    addPointsToHTML(xVals, yVals, ctrlXVals, ctrlYVals);
  }
}

async function savePath(e) {
  var selectedPath = await saveFile()

  await invoke('save_file', {path: selectedPath});
}

document.getElementById("add-item-type").onchange = function () {

  if (document.getElementById("add-item-type").value == "lateral") {
    const endTolInput = document.getElementById("end-tolerance-input");
    endTolInput.style.left = "45%";
    endTolInput.style.width = "20%";
    // endTolInput.style.diplay = ""

    const timeoutInput = document.getElementById("timeout-input");
    timeoutInput.style.left = "70%";
    timeoutInput.style.width = "25%";
    // timeoutInput.style.diplay = ""

    const specificInput = document.getElementById("specific-val-input");
    specificInput.placeholder = "Distance";
    specificInput.style.left = "5%";
    specificInput.style.width = "35%";
  }
  else if (document.getElementById("add-item-type").value == "turn") {
    const endTolInput = document.getElementById("end-tolerance-input");
    endTolInput.style.left = "45%";
    endTolInput.style.width = "20%";
    // endTolInput.style.diplay = ""

    const timeoutInput = document.getElementById("timeout-input");
    timeoutInput.style.left = "70%";
    timeoutInput.style.width = "25%";
    // timeoutInput.style.diplay = ""

    const specificInput = document.getElementById("specific-val-input");
    specificInput.placeholder = "Angle";
    specificInput.style.left = "5%";
    specificInput.style.width = "35%";
  } 
  else if (document.getElementById("add-item-type").value == "command") {
    const endTolInput = document.getElementById("end-tolerance-input");
    endTolInput.style.left = "45%";
    endTolInput.style.width = "20%";
    // endTolInput.style.display = "none";

    const timeoutInput = document.getElementById("timeout-input");
    timeoutInput.style.left = "70%";
    timeoutInput.style.width = "25%";
    // timeoutInput.style.display = "none";

    const specificInput = document.getElementById("specific-val-input");
    specificInput.placeholder = "Command Name";
    specificInput.style.left = "5%";
    specificInput.style.width = "90%";
  }
}

listen('export-backend', async (e) => {
  exportPath(e)
})

listen('new-backend', async (e) => {
  clearPath(e)
})

listen('save-backend', async (e) => {
  savePath(e)
})

listen('load-backend', async (e) => {
  loadPath(e)
})

listen('export_mvmt-backend', async (e) => {
  var selectedPath = await saveFile()

  await invoke('export_movement', {path: selectedPath, id: selected_mvmt});
})

var field_type = 0; // 0 for match, 1 for skills

listen('change_field-backend', async (e) => {
  field_img = document.getElementById("field");
  
  if (field_type == 1) {
    field_img.style.background = "url('his_match.png') no-repeat center center / cover"
    field_type = 0;
  } else {
    field_img.style.background = "url('his_skills.png') no-repeat center center / cover"
    field_type = 1;
  }
})


window.addEventListener("DOMContentLoaded", () => {
  field = document.getElementById("field")
  field.addEventListener("mousedown", (e) => {
    e.preventDefault();
    detectClick(e);
  })

  document.getElementById("add-item-menu").addEventListener('submit', (e) => {
    e.preventDefault();

    addRouteToControl(e);

    return false;
  })

  document.getElementById("configure-item-menu").addEventListener('submit', (e) =>{
    e.preventDefault();

    return false;
  })

  document.getElementById("edit-start-pos").addEventListener('submit', (e) =>{
    e.preventDefault();

    changeStartPos();

    return false;
  })

  document.addEventListener("mousemove" , (e) => {
    moveControlPoint(e, e.clientX, e.clientY);
  })

  document.addEventListener("mouseup", (e) => {
    e.preventDefault();
    deselectCtrl(e);
  })

  document.getElementById("delete_button").addEventListener("mousedown", (e) => {
    e.preventDefault();

    deleteLastMvmt(e);
  })

  document.getElementById("change-item-submit").addEventListener("mousedown", (e) => {
    e.preventDefault();

    updateSelectedRoute(e);
  })
});
