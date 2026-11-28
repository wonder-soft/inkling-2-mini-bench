from typing import List

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
    app = FastAPI(title="Tasks API")
    tasks: dict[int, Task] = {}
    next_id = 1

    def get_task_or_404(task_id: int) -> Task:
        task = tasks.get(task_id)
        if task is None:
            raise HTTPException(status_code=404, detail="Task not found")
        return task

    @app.get("/health")
    def health():
        return {"status": "ok"}

    @app.get("/tasks", response_model=List[Task])
    def list_tasks():
        return [tasks[task_id] for task_id in sorted(tasks)]

    @app.post("/tasks", response_model=Task, status_code=201)
    def create_task(payload: TaskCreate):
        nonlocal next_id
        task = Task(id=next_id, title=payload.title, done=False)
        tasks[next_id] = task
        next_id += 1
        return task

    @app.get("/tasks/{task_id}", response_model=Task)
    def get_task(task_id: int):
        return get_task_or_404(task_id)

    @app.put("/tasks/{task_id}", response_model=Task)
    def update_task(task_id: int, payload: TaskUpdate):
        get_task_or_404(task_id)
        updated = Task(id=task_id, title=payload.title, done=payload.done)
        tasks[task_id] = updated
        return updated

    @app.delete("/tasks/{task_id}", status_code=204)
    def delete_task(task_id: int):
        get_task_or_404(task_id)
        del tasks[task_id]
        return None

    return app


app = create_app()
