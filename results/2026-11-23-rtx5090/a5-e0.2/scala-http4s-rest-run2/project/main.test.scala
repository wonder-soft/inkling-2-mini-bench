import cats.effect.IO
import io.circe.Json
import io.circe.generic.auto._
import io.circe.syntax._
import munit.CatsEffectSuite
import org.http4s._
import org.http4s.circe._

class MainSuite extends CatsEffectSuite {

  private def get(app: HttpApp[IO], path: String): IO[Response[IO]] =
    app.run(Request[IO](Method.GET, Uri.unsafeFromString(path)))

  test("GET /health returns 200") {
    freshApp.flatMap { app =>
      get(app, "/health").flatMap { resp =>
        assertEquals(resp.status, Status.Ok)
      }
    }
  }

  test("POST /tasks returns 201 and id 1") {
    freshApp.flatMap { app =>
      val req = Request[IO](Method.POST, Uri.unsafeFromString("/tasks"))
        .withEntity(Json.obj("title" -> "Buy milk".asJson))
      app.run(req).flatMap { resp =>
        assertEquals(resp.status, Status.Created)
        resp.as[Task].flatMap { t =>
          assertEquals(t.id, 1L)
          assertEquals(t.title, "Buy milk")
          assertEquals(t.done, false)
        }
      }
    }
  }

  test("GET /tasks/1 after creation returns the task") {
    freshApp.flatMap { app =>
      val req = Request[IO](Method.POST, Uri.unsafeFromString("/tasks"))
        .withEntity(Json.obj("title" -> "Write tests".asJson))
      app.run(req).flatMap { _ =>
        get(app, "/tasks/1").flatMap { resp =>
          assertEquals(resp.status, Status.Ok)
          resp.as[Task].flatMap { t =>
            assertEquals(t.id, 1L)
            assertEquals(t.title, "Write tests")
          }
        }
      }
    }
  }

  test("GET /tasks/999 returns 404") {
    freshApp.flatMap { app =>
      get(app, "/tasks/999").flatMap { resp =>
        assertEquals(resp.status, Status.NotFound)
      }
    }
  }

  test("DELETE an existing task returns 204, then GET returns 404") {
    freshApp.flatMap { app =>
      val req = Request[IO](Method.POST, Uri.unsafeFromString("/tasks"))
        .withEntity(Json.obj("title" -> "Delete me".asJson))
      app.run(req).flatMap { _ =>
        val del = Request[IO](Method.DELETE, Uri.unsafeFromString("/tasks/1"))
        app.run(del).flatMap { delResp =>
          assertEquals(delResp.status, Status.NoContent)
          get(app, "/tasks/1").flatMap { getResp =>
            assertEquals(getResp.status, Status.NotFound)
          }
        }
      }
    }
  }
}
