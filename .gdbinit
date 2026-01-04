set pagination off
set print pretty on

# Enable FreeRTOS thread awareness
define hook-stop
  info threads
end

# Helper to show all FreeRTOS tasks
define show-freertos-tasks
  printf "Current task: %s\n", (char*)((TCB_t*)pxCurrentTCB)->pcTaskName
  printf "Number of tasks: %d\n", uxCurrentNumberOfTasks
end

document show-freertos-tasks
Show FreeRTOS tasks information
end
