# architecture

## "server + workers" instead of just "server".

Initially, the implementation started with just a `server` component. Frequently "modular monoliths" work well to simplify
the system, however, it did not work well in this case. The main problem is that the main binary had to include some heavy
dependencies like `libtorch` (probably with gpu support) and, as the result, the size of the docker image is quite big,
it gets more difficult to build and deploy the application.

It seems that splitting everything into two components would simplify the architecture a bit:
- `server` - lightweight, no dynamic dependencies. Processes API calls, performs scheduling, etc.
- `worker` - runs the actual models on the GPU, contains all necessary dependencies, communicates with `server`.

Alternative would probably be `worker` that runs specialized "subworkers" that can process one type of model each. It is
probably a good idea to keep this option in mind, but do not implement it until there are reasons to (currently there is
no visible benefit to doing something like that).

## workers pull tasks from the server, instead of push

This is because workers may be running on hardware without externally-reachable IP address (for example, at home).

## workers pull tasks by API, not by accessing database or queue directly

For cleaner architecture and because workers may be running on hardware that we cannot trust.

## autoscaling

Currently will be implemented in the simplest way possible: 0 gpu workers -> 1 gpu worker on GCP. The worker will
automatically shutdown after being idle for X minutes. The server is automatically started if it is not running
and there has just been a task submitted.

## peeking task for worker

Currently, FIFO queue. Better priority-based algorithms in the future.

## implementation

Keep is simple currently. Less generic stuff, aim to get image generation model running with the new architecture.
Save results to object storage bucket.

## live logs

It is important to keep user updated with the current progress, as it may take a while to generate the output. That can be
implemented by gRPC streaming to the client and a Redis instance for broadcasting.