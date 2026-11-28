import cats.effect.IO
import io.circe.generic.auto._
import munit.CatsEffectSuite
import org.http4s._
import org.http4s.circe._

class ApiSuite extends CatsEffectSuite {

  private def get(app: HttpApp[IO], path: String): IO[Response[IO]] =
    app.run(Request[IO](Method.GET, Uri.unsafeFromString(path)))

  private def post(app: HttpApp[IO], path: String, body: String): IO[Response[IO]] =
    app.run(Request[IO](Method.POST, Uri.unsafeFromString(path)).withEntity(body))

  test("GET /health returns 200") {
    freshApp.flatMap { app =>
      get(app, "/health").flatMap { res =>
        IO(assertEquals(res.status, Status.Ok))
      }
    }
  }

  test("POST /tasks returns 201 and id 1") {
    freshApp.flatMap { app =>
      post(app, "/tasks", """{"title":"Buy milk"}""").flatMap { res =>
        res.as[Task].flatMap { task =>
          IO(assertEquals(res.status, Status.Created)) *>
          IO(assertEquals(task.id, 1L)) *>
          IO(assertEquals(task.title, "Buy milk")) *>
          IO(assertEquals(task.done, false))
        }
      }
    }
  }

  test("GET /tasks/1 after creation returns the task") {
    freshApp.flatMap { app =>
      post(app, "/tasks", """{"title":"Buy milk"}""").flatMap { _ =>
        get(app, "/tasks/1")
      }.flatMap { res =>
        res.as[Task].flatMap { task =>
          IO(assertEquals(res.status, Status.Ok)) *>
          IO(assertEquals(task.id, 1L)) *>
          IO(assertEquals(task.title, "Buy milk")) *>
          IO(assertEquals(task.done, false))
        }
      }
    }
  }

  test("GET /tasks/999 returns 404") {
    freshApp.flatMap { app =>
      get(app, "/tasks/999").flatMap { res =>
        IO(assertEquals(res.status, Status.NotFound))
      }
    }
  }

  test("DELETE an existing task returns 204, and a subsequent GET returns 404") {
    freshApp.flatMap { app =>
      post(app, "/tasks", """{"title":"Buy milk"}""").flatMap { _ =>
        app.run(Request[IO](Method.DELETE, Uri.unsafeFromString("/tasks/1")))
      }.flatMap { res =>
        IO(assertEquals(res.status, Status.NoContent)) *>
        get(app, "/tasks/1").flatMap { res2 =>
          IO(assertEquals(res2.status, Status.NotFound))
        }
      }
    }
  }
}
