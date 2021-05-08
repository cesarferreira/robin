#!/usr/bin/env node
"use strict";

const Chalk = require("chalk");
const Utils = require("./utils/utils");
const log = console.log;

const InitTask = require("./tasks/init_task");
const ListTask = require("./tasks/list_task");

// Main code //
const self = (module.exports = {
  init: (input, flags) => {
    const command = input[0];
    const params = input.subarray(1, input.length);

	const availableComands = ListTask.readConfigFile();

    switch (command.toLowerCase()) {
      case "init":
		  // TODO if file exists, REFUSE TO do it
        InitTask.init(params);
        break;
      case "list":
        ListTask.init(params);
        break;

      default:
		let a = availableComands.find((a)=>{
			a.tit
		})

		// if i can find it tell run it, other wise say the following
        log(`Sorry, cant find ${command} with `);
    }
  },
});
