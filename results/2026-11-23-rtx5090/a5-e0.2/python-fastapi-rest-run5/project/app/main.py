from fastapi import FastAPI, HTTPException, Response, status
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

    @app.get("/tasks", response_model=list[Task])
    def list_tasks():
        return sorted(tasks.values(), key=lambda t: t.id)

    @app.post("/tasks", response_model=Task, status_code=status.HTTP_201_CREATED)
    def create_task(payload: TaskCreate):
        task = Task(id=state["next_id"], title=payload.title, done=False)
        state["next_id"] += 1
        tasks[task.id] = task
        return task

    @app.get("/tasks/{task_id}", response_model=Task)
    def get_task(task_id: int):
        task = tasks.get(task_id)
        if task is None:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
        return task

    @app.put("/tasks/{task_id}", response_model=Task)
    def update_task(task_id: int, payload: TaskUpdate):
        task = tasks.get(task_id)
        if task is None:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
        updated = Task(id=task.id, title=payload.title, done=payload.done)
        tasks[task_id] = updated
        return updated

    @app.delete("/tasks/{task_id}", status_code=status.HTTP_204_NO_CONTENT)
    def delete_task(task_id: int):
        if task_id not in tasks:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Task not found")
        del tasks[task_id]
        return Response(status_code=status.HTTP_204_NO_CONTENT)

    return app


app = create_app()
