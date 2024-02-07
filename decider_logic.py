#!/usr/bin/python3

import sys
import json
import os

input_str = sys.argv[1]


log_json = json.loads(input_str)

custom_logic_file = "custom_logic.py"
if os.path.isfile(custom_logic_file):
    try:
        custom_logic_module = __import__(custom_logic_file[:-3])
        custom_logic_module.custom_logic(log_json)
    except ImportError:
        print(f"Error importing {custom_logic_file}")
        default_logic(log_json)
else:
    default_logic(log_json)

def default_logic(log_json):
    if log_json["action"] == "run":
      exit_code = int(log_json["exit_code"])
      #If it exit with nothing wrong we may try to restore first
      if exit_code == 0:
        dec = 1
      #If Exit occur by some signal(This also include the checkpoint scenario), we will continue to standby mode
      elif exit_code >= 128 and exit_code <= 159:
        dec = 2
      #Else something can be wrong, we will start it from scratch.
      else:
        dec = 0

      file = open('/decider.txt', 'w')
      file.write(str(dec))
      file.close()
