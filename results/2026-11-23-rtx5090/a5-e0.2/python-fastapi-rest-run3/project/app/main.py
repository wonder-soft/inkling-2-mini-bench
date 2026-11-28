from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field


class TaskCreate(BaseModel):
    title: str = Field(min_length=1)


class TaskUpdate(BaseModel):
    title: str = Field(min_length=1)
    done: bool


class Task(BaseModel):
    id: int
    title: str
    done: bool


def create_app() -> FastAPI:
    app = FastAPI()
    tasks: dict[int, Task] = {}
    next_id = 1

    @app.get("/health")
    def health():
        return {"status": "ok"}

    @app.get("/tasks")
    def list_tasks():
        return [tasks[i] for i in sorted(tasks)]

    @app.post("/tasks", status_code=201)
    def create_task(payload: TaskCreate):
        nonlocal next_id
        task = Task(id=next_id, title=payload.title, done=False)
        tasks[next_id] = task
        next_id += 1
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
        task = Task(id=task_id, title=payload.title, done=payload.done)
        tasks[task_id] = task
        return task

    @app.delete("/tasks/{task_id}", status_code=204)
    def delete_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=404, detail="Task not found")
        del tasks[task_id]
        return None

    return app


app = create_app()
