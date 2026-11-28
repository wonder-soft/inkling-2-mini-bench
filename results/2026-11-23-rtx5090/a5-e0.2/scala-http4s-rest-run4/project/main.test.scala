import cats.effect.unsafe.implicits.*
import io.circe.Json
import munit.FunSuite
import org.http4s.*
import org.http4s.circe.*
import org.http4s.implicits.*

class MainSuite extends FunSuite {

  test("GET /health returns 200") {
    freshApp.flatMap { app =>
      app.run(Request[cats.effect.IO](Method.GET, uri"/health")).flatMap { resp =>
        assertEquals(resp.status, Status.Ok)
        resp.as[Json].map(_ => ())
      }
    }.unsafeRunSync()
  }

  test("POST /tasks returns 201 and id 1") {
    freshApp.flatMap { app =>
      val req = Request[cats.effect.IO](Method.POST, uri"/tasks")
        .withEntity(Json.obj("title" -> Json.fromString("Buy milk")))
      app.run(req).flatMap { resp =>
        assertEquals(resp.status, Status.Created)
        resp.as[Task].map { task =>
          assertEquals(task.id, 1L)
          assertEquals(task.title, "Buy milk")
          assertEquals(task.done, false)
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/1 after creation returns the task") {
    freshApp.flatMap { app =>
      val create = Request[cats.effect.IO](Method.POST, uri"/tasks")
        .withEntity(Json.obj("title" -> Json.fromString("Write tests")))
      app.run(create).flatMap { _ =>
        app.run(Request[cats.effect.IO](Method.GET, uri"/tasks/1")).flatMap { resp =>
          assertEquals(resp.status, Status.Ok)
          resp.as[Task].map { task =>
            assertEquals(task.id, 1L)
            assertEquals(task.title, "Write tests")
          }
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/999 returns 404") {
    freshApp.flatMap { app =>
      app.run(Request[cats.effect.IO](Method.GET, uri"/tasks/999")).flatMap { resp =>
        assertEquals(resp.status, Status.NotFound)
      }
    }.unsafeRunSync()
  }

  test("DELETE existing task returns 204, subsequent GET returns 404") {
    freshApp.flatMap { app =>
      val create = Request[cats.effect.IO](Method.POST, uri"/tasks")
        .withEntity(Json.obj("title" -> Json.fromString("Delete me")))
      app.run(create).flatMap { _ =>
        app.run(Request[cats.effect.IO](Method.DELETE, uri"/tasks/1")).flatMap { resp =>
          assertEquals(resp.status, Status.NoContent)
          app.run(Request[cats.effect.IO](Method.GET, uri"/tasks/1")).flatMap { getResp =>
            assertEquals(getResp.status, Status.NotFound)
          }
        }
      }
    }.unsafeRunSync()
  }
}
