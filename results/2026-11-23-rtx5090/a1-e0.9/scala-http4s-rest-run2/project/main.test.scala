import cats.effect.IO
import org.http4s._
import org.http4s.circe._
import io.circe._
import io.circe.generic.auto._
import io.circe.syntax._
import munit.FunSuite

class MainSuite extends FunSuite {

  private def postReq(title: String): Request[IO] =
    Request[IO](Method.POST, Uri.unsafeFromString("/tasks"))
      .withEntity(Json.obj("title" -> title.asJson))

  test("GET /health returns 200") {
    freshApp.flatMap { app =>
      app.run(Request[IO](Method.GET, Uri.unsafeFromString("/health"))).flatMap { resp =>
        assertEquals(resp.status, Status.Ok)
        resp.as[Json].map { json =>
          assertEquals(json.hcursor.get[String]("status"), Right("ok"))
        }
      }
    }.unsafeRunSync()
  }

  test("POST /tasks returns 201 and id 1") {
    freshApp.flatMap { app =>
      app.run(postReq("Buy milk")).flatMap { resp =>
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
      app.run(postReq("Buy milk")).flatMap { _ =>
        app.run(Request[IO](Method.GET, Uri.unsafeFromString("/tasks/1"))).flatMap { resp =>
          assertEquals(resp.status, Status.Ok)
          resp.as[Task].map { task =>
            assertEquals(task.id, 1L)
            assertEquals(task.title, "Buy milk")
            assertEquals(task.done, false)
          }
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/999 returns 404") {
    freshApp.flatMap { app =>
      app.run(Request[IO](Method.GET, Uri.unsafeFromString("/tasks/999"))).map { resp =>
        assertEquals(resp.status, Status.NotFound)
      }
    }.unsafeRunSync()
  }

  test("DELETE an existing task returns 204, then GET returns 404") {
    freshApp.flatMap { app =>
      app.run(postReq("Buy milk")).flatMap { _ =>
        app.run(Request[IO](Method.DELETE, Uri.unsafeFromString("/tasks/1"))).flatMap { resp =>
          assertEquals(resp.status, Status.NoContent)
          app.run(Request[IO](Method.GET, Uri.unsafeFromString("/tasks/1"))).map { resp2 =>
            assertEquals(resp2.status, Status.NotFound)
          }
        }
      }
    }.unsafeRunSync()
  }

  test("PUT /tasks/1 updates the task") {
    freshApp.flatMap { app =>
      app.run(postReq("Buy milk")).flatMap { _ =>
        val putReq = Request[IO](Method.PUT, Uri.unsafeFromString("/tasks/1"))
          .withEntity(Json.obj("title" -> "Buy oat milk".asJson, "done" -> true.asJson))
        app.run(putReq).flatMap { resp =>
          assertEquals(resp.status, Status.Ok)
          resp.as[Task].map { task =>
            assertEquals(task.id, 1L)
            assertEquals(task.title, "Buy oat milk")
            assertEquals(task.done, true)
          }
        }
      }
    }.unsafeRunSync()
  }
}
