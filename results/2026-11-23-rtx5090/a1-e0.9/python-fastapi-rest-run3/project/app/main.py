from fastapi import FastAPI, HTTPException
from pydantic import BaseModel


class TaskCreate(BaseModel):
    title: str


class TaskUpdate(BaseModel):
    title: str
    done: bool


class Task(BaseModel):
    id: int
    title: str
    done: bool


def create_app() -> FastAPI:
    app = FastAPI()

    tasks: dict[int, Task] = {}
    state = {"next_id": 1}

    @app.get("/health")
    def health():
        return {"status": "ok"}

    @app.get("/tasks")
    def list_tasks():
        return [tasks[task_id] for task_id in sorted(tasks)]

    @app.post("/tasks", status_code=201)
    def create_task(payload: TaskCreate):
        task = Task(id=state["next_id"], title=payload.title, done=False)
        state["next_id"] += 1
        tasks[task.id] = task
        return task

    @app.get("/tasks/{task_id}")
    def get_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=404, detail="Task not found")
        return tasks[task_id]

    @app.put("/tasks/{task_id}")
    def update_task(task_id: int, payload: TaskUpdate):
        if task_id not in tasks:
            raise HTTPException(status_code=404, detail="Task not found")
        tasks[task_id] = Task(id=task_id, title=payload.title, done=payload.done)
        return tasks[task_id]

    @app.delete("/tasks/{task_id}", status_code=204)
    def delete_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=404, detail="Task not found")
        del tasks[task_id]
        return None

    return app


app = create_app()
