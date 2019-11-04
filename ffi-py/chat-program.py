#!/usr/bin/env python3

import sys
import os

# Add dindex to the library search path; this is not necessary if
# dindex is already on your OS library path.
sys.path.append(os.path.abspath("../target/release/"))

import libdindex as dindex

# Used for UI
from threading import Thread
import tkinter # TODO ugh see https://medium.com/swlh/lets-write-a-chat-app-in-python-f6783a9ac170

CONNECTING_USERS_QUERY = dindex.record({
  "action": "(?i)connect", # case insensitive match of "connect"
  "username": ".*",
})
MESSAGE_QUERY = dindex.record({
  "action": "(?i)msg",
  "username": ".*",
  "message": ".*",
})
LEAVING_USERS_QUERY = dindex.record({
  "action": "(?i)leaving",
  "username": ".*",
})
ANY_ACTION_QUERY = dindex.record({
  "action": "(?i)leaving|connect|msg",
  "username": ".*",
});


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
  dindex.client_publish_sync(config, dindex.record({
    "action": "connect",
    "username": str(username),
  }))

def publish_msg_rec(config, username, msg):
  dindex.client_publish_sync(config, dindex.record({
    "action": "msg",
    "username": str(username),
    "message": str(msg),
  }))
  

def publish_leaving_rec(config, username):
  dindex.client_publish_sync(config, dindex.record({
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
  WINDOW_CLOSED = False # used to signal rust code to exit
  try:
    
    def on_window_close(event=None):
      global WINDOW_CLOSED
      print("Publishing leaving record from UI exit")
      WINDOW_CLOSED = True
      publish_leaving_rec(config, our_username)
      sys.exit(0)
    
    # Create UI first
    top = tkinter.Tk()
    top.title("chat-program")
    
    messages_frame = tkinter.Frame(top)
    my_msg = tkinter.StringVar()  # For the messages to be sent.
    my_msg.set("Type your messages here.")
    scrollbar = tkinter.Scrollbar(messages_frame)  # To navigate through past messages.
    
    msg_list = tkinter.Listbox(messages_frame, height=15, width=50, yscrollcommand=scrollbar.set)
    scrollbar.pack(side=tkinter.RIGHT, fill=tkinter.Y)
    msg_list.pack(side=tkinter.LEFT, fill=tkinter.BOTH)
    msg_list.pack()
    messages_frame.pack()
    
    def on_user_enter(event=None):
      msg = my_msg.get()
      my_msg.set("")  # Clears input field.
      publish_msg_rec(config, our_username, msg)
      # Should not be necessary
      msg_list.insert(tkinter.END, "{}: {}".format(our_username, msg))
    
    entry_field = tkinter.Entry(top, textvariable=my_msg)
    entry_field.bind("<Return>", on_user_enter)
    entry_field.pack()
    send_button = tkinter.Button(top, text="Send", command=on_user_enter)
    send_button.pack()
    top.protocol("WM_DELETE_WINDOW", on_window_close)
    
    # Create new thread to listen to incoming chat data
    def incoming_chat_data_handler():
      def on_any_action(rec):
        global WINDOW_CLOSED
        
        if WINDOW_CLOSED:
          return "EndListen"
        if not bool(rec):
          # Empty record, ignore
          return "Continue"
        else:
          if rec["action"] and "connect" in rec["action"]:
            print("User {} connected!".format(rec["username"]))
            active_users.append(rec)
            active_users = dedupe_results(active_users)
            msg_list.insert(tkinter.END, "{} joined the chat".format(rec["username"]))
            
          elif rec["action"] and "leaving" in rec["action"]:
            print("User {} disconnected!".format(rec["username"]))
            active_users = [x for x in active_users if x["username"] != rec["username"] ]
            active_users = dedupe_results(active_users)
            msg_list.insert(tkinter.END, "{} left the chat".format(rec["username"]))
            
          else:
            print("User {} said: {}".format(rec["username"], rec["message"]))
            msg_list.insert(tkinter.END, "{}: {}".format(rec["username"], rec["message"]))
            
          return "Continue"
      
      print("Listening...")
      dindex.client_listen_sync_with_timer(config, ANY_ACTION_QUERY, 250, on_any_action)
    
    receive_thread = Thread(target=incoming_chat_data_handler)
    receive_thread.start()
    
    tkinter.mainloop()  # Starts GUI execution.
    
  except Exception as e:
    print(e)
    publish_leaving_rec(config, our_username)
  
  print("Main thread ends")
  
  
