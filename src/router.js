#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const Utils = require("./utils/utils");
const log = console.log;

const InitTask = require("./tasks/init_task");
const ListTask = require("./tasks/list_task");
const RunTask = require("./tasks/run_task");

function interruptIfConfigMissing() {
  if (!Utils.configFileExists()) {
    Utils.titleError(`${Utils.CONFIG_FILE_NAME} file is missing, please use the 'init' command` );
    process.exit();
  }
}

// Main code //
const self = (module.exports = {
  init: (input, flags) => {
    const command = input.join(" ");
    const params = input.subarray(1, input.length);

    // log(input.join(" "))
    const availableCommands = ListTask.getCommandList();
    
    switch (command.toLowerCase()) {
      case "init":
        // TODO if file exists, REFUSE TO do it
        InitTask.init(params);
        break;
        case "list":
        interruptIfConfigMissing();
        // ListTask.init(params);
        break;

      default:
		const result = RunTask.find(command, availableCommands);
		
		if (result.length != 0 ) {
			RunTask.run(result[0], flags);
		} else {
			log(`Sorry, cant find "${command}" in your `);
		}
			
    }
  },
});
