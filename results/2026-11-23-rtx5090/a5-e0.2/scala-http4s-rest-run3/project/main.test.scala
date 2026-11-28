import cats.effect.IO
import io.circe.Json
import io.circe.syntax._
import org.http4s._
import org.http4s.implicits._

class MainSuite extends munit.FunSuite {

  test("GET /health returns 200") {
    freshApp.flatMap { app =>
      val req = Request[IO](Method.GET, uri"/health")
      app.run(req).flatMap { resp =>
        assertEquals(resp.status, Status.Ok)
      }
    }.unsafeRunSync()
  }

  test("POST /tasks returns 201 and id 1") {
    freshApp.flatMap { app =>
      val req = Request[IO](Method.POST, uri"/tasks")
        .withEntity(Json.obj("title" -> "Buy milk".asJson))
      app.run(req).flatMap { resp =>
        assertEquals(resp.status, Status.Created)
        resp.as[Task].flatMap { task =>
          assertEquals(task.id, 1L)
          assertEquals(task.title, "Buy milk")
          assertEquals(task.done, false)
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/1 after creation returns the task") {
    freshApp.flatMap { app =>
      val post = Request[IO](Method.POST, uri"/tasks")
        .withEntity(Json.obj("title" -> "Write tests".asJson))
      app.run(post).flatMap { _ =>
        val get = Request[IO](Method.GET, uri"/tasks/1")
        app.run(get).flatMap { resp =>
          assertEquals(resp.status, Status.Ok)
          resp.as[Task].flatMap { task =>
            assertEquals(task.id, 1L)
            assertEquals(task.title, "Write tests")
          }
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/999 returns 404") {
    freshApp.flatMap { app =>
      val req = Request[IO](Method.GET, uri"/tasks/999")
      app.run(req).flatMap { resp =>
        assertEquals(resp.status, Status.NotFound)
      }
    }.unsafeRunSync()
  }

  test("DELETE existing task returns 204, subsequent GET returns 404") {
    freshApp.flatMap { app =>
      val post = Request[IO](Method.POST, uri"/tasks")
        .withEntity(Json.obj("title" -> "Delete me".asJson))
      app.run(post).flatMap { _ =>
        val del = Request[IO](Method.DELETE, uri"/tasks/1")
        app.run(del).flatMap { delResp =>
          assertEquals(delResp.status, Status.NoContent)
          val get = Request[IO](Method.GET, uri"/tasks/1")
          app.run(get).flatMap { getResp =>
            assertEquals(getResp.status, Status.NotFound)
          }
        }
      }
    }.unsafeRunSync()
  }
}
