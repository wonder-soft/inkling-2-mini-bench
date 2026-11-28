import cats.effect.IO
import org.http4s._
import org.http4s.implicits._
import org.http4s.circe._
import io.circe._
import io.circe.syntax._
import munit.FunSuite

class MainSuite extends FunSuite {

  private def postTask(app: HttpApp[IO], title: String): IO[Response[IO]] = {
    val req = Request[IO](Method.POST, uri"/tasks")
      .withContentType(`application/json`)
      .withEntity(json"""{"title":"$title"}""")
    app.run(req)
  }

  test("GET /health returns 200") {
    freshApp.flatMap { app =>
      app.run(Request[IO](Method.GET, uri"/health")).flatMap { resp =>
        IO(assertEquals(resp.status, Status.Ok))
      }
    }.unsafeRunSync()
  }

  test("POST /tasks returns 201 and id 1") {
    freshApp.flatMap { app =>
      postTask(app, "Buy milk").flatMap { resp =>
        resp.as[Task].flatMap { task =>
          IO(
            assertEquals(resp.status, Status.Created)
            assertEquals(task.id, 1L)
            assertEquals(task.title, "Buy milk")
            assertEquals(task.done, false)
          )
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/1 after creation returns the task") {
    freshApp.flatMap { app =>
      postTask(app, "Test task").flatMap { _ =>
        app.run(Request[IO](Method.GET, uri"/tasks/1")).flatMap { resp =>
          resp.as[Task].flatMap { task =>
            IO(
              assertEquals(resp.status, Status.Ok)
              assertEquals(task.id, 1L)
              assertEquals(task.title, "Test task")
              assertEquals(task.done, false)
            )
          }
        }
      }
    }.unsafeRunSync()
  }

  test("GET /tasks/999 returns 404") {
    freshApp.flatMap { app =>
      app.run(Request[IO](Method.GET, uri"/tasks/999")).flatMap { resp =>
        IO(assertEquals(resp.status, Status.NotFound))
      }
    }.unsafeRunSync()
  }

  test("DELETE existing task returns 204, subsequent GET returns 404") {
    freshApp.flatMap { app =>
      postTask(app, "Bye").flatMap { _ =>
        app.run(Request[IO](Method.DELETE, uri"/tasks/1")).flatMap { resp =>
          IO(assertEquals(resp.status, Status.NoContent)).flatMap { _ =>
            app.run(Request[IO](Method.GET, uri"/tasks/1")).flatMap { resp2 =>
              IO(assertEquals(resp2.status, Status.NotFound))
            }
          }
        }
      }
    }.unsafeRunSync()
  }
}
