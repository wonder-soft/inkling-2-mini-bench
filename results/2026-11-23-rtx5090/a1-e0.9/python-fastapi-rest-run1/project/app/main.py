from fastapi import FastAPI, HTTPException, status
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
    next_id = 1

    @app.get("/health")
    def health():
        return {"status": "ok"}

    @app.get("/tasks")
    def list_tasks():
        return [tasks[task_id] for task_id in sorted(tasks)]

    @app.post("/tasks", status_code=status.HTTP_201_CREATED)
    def create_task(payload: TaskCreate):
        nonlocal next_id
        task = Task(id=next_id, title=payload.title, done=False)
        tasks[next_id] = task
        next_id += 1
        return task

    @app.get("/tasks/{task_id}")
    def get_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
        return tasks[task_id]

    @app.put("/tasks/{task_id}")
    def update_task(task_id: int, payload: TaskUpdate):
        if task_id not in tasks:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
        task = tasks[task_id]
        task.title = payload.title
        task.done = payload.done
        return task

    @app.delete("/tasks/{task_id}", status_code=status.HTTP_204_NO_CONTENT)
    def delete_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
        del tasks[task_id]
        return None

    return app


app = create_app()
