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
    tasks: dict[int, dict] = {}
    counter = {"next": 1}

    @app.get("/health")
    def health():
        return {"status": "ok"}

    @app.get("/tasks", response_model=list[Task])
    def list_tasks():
        return [tasks[i] for i in sorted(tasks)]

    @app.post("/tasks", response_model=Task, status_code=201)
    def create_task(payload: TaskCreate):
        task_id = counter["next"]
        counter["next"] += 1
        task = {"id": task_id, "title": payload.title, "done": False}
        tasks[task_id] = task
        return task

    @app.get("/tasks/{task_id}", response_model=Task)
    def get_task(task_id: int):
        task = tasks.get(task_id)
        if task is None:
            raise HTTPException(status_code=404, detail="Task not found")
        return task

    @app.put("/tasks/{task_id}", response_model=Task)
    def update_task(task_id: int, payload: TaskUpdate):
        task = tasks.get(task_id)
        if task is None:
            raise HTTPException(status_code=404, detail="Task not found")
        task["title"] = payload.title
        task["done"] = payload.done
        return task

    @app.delete("/tasks/{task_id}", status_code=204)
    def delete_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=404, detail="Task not found")
        del tasks[task_id]
        return None

    return app


app = create_app()
