#!/usr/bin/env python3

import sys
import os

# Add dindex to the library search path; this is not necessary if
# dindex is already on your OS library path.
sys.path.append(os.path.abspath("../target/release/"))

import libdindex as dindex

CONNECTING_USERS_QUERY = dindex.record({
  "action": "(?i)connect", # case insensitive match of "connect"
  "username": ".*"
})
LEAVING_USERS_QUERY = dindex.record({
  "action": "(?i)leaving",
  "username": ".*"
})


# Turns [{"a":"a"},{"b":"b"},{"a":"a"}] into [{"a":"a"},{"b":"b"}]
def dedupe_results(results_list):
  res_list = []
  for i in range(len(results_list)):
      if results_list[i] not in results_list[i + 1:]:
          res_list.append(results_list[i])
  return res_list

def get_actively_connected_users(config):
  all_connected_users = dindex.client_query_sync(config, CONNECTING_USERS_QUERY)
  all_disconnected_users = dindex.client_query_sync(config, LEAVING_USERS_QUERY)

  all_connected_users = dedupe_results(all_connected_users)
  all_disconnected_users = dedupe_results(all_disconnected_users)

  all_users = []
  for connected in all_connected_users:
    is_connected = True
    for disconn in all_disconnected_users:
      if disconn["username"] == connected["username"]:
        is_connected = False
        break
    
    if is_connected:
      all_users.append(connected)
      
  return all_users

def publish_connect_rec(config, username):
  dindex.client_publish_sync(dindex.record({
    "action": "connect",
    "username": str(username),
  }))

def publish_leaving_rec(config, username):
  dindex.client_publish_sync(dindex.record({
    "action": "leaving",
    "username": str(username),
  }))

if __name__ == '__main__':
  config = dindex.config()
  
  our_username = os.environ["USER"]
  if not our_username:
    our_username = "Unknown Username"
  print("Our username is {}".format(our_username))

  print("Connecting to the following servers:")
  for server in config["servers"]:
    print(server["name"]+" over "+server["protocol"]+" on host "+server["host"])
  
  active_users = get_actively_connected_users(config)
  print("{} active users: {}".format(len(active_users), active_users))
  
  publish_connect_rec(config, our_username)
  try:
    
    def on_new_user(rec):
      print("User {} connected!".format(rec["username"]))
      active_users.append(rec)
      active_users = dedupe_results(active_users)
      return "EndListen"
    
    print("Listening...")
    dindex.client_listen_sync(config, CONNECTING_USERS_QUERY, on_new_user)
    
  except e:
    print(e)
  
  publish_leaving_rec(config, our_username)
  
  
