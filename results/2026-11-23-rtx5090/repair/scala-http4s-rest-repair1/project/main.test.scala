import cats.effect.IO
import org.http4s.{HttpApp, Method, Request, Status, Uri}
import org.http4s.circe.CirceEntityCodec._
import io.circe.generic.auto._
import io.circe.syntax._

class MainSuite extends munit.FunSuite {
  private def newApp(): HttpApp[IO] = freshApp.unsafeRunSync()

  private def get(app: HttpApp[IO], path: String) =
    app.run(Request[IO](Method.GET, Uri.unsafeFromString(path))).unsafeRunSync()

  private def post(app: HttpApp[IO], path: String, title: String) =
    app.run(
      Request[IO](Method.POST, Uri.unsafeFromString(path))
        .withEntity(io.circe.Json.obj("title" -> io.circe.Json.fromString(title)))
    ).unsafeRunSync()

  test("GET /health returns 200") {
    val app = newApp()
    val res = get(app, "/health")
    assertEquals(res.status, Status.Ok)
    val body = res.as[io.circe.Json].unsafeRunSync()
    assertEquals(body.hcursor.get[String]("status").toOption, Some("ok"))
  }

  test("POST /tasks returns 201 and id 1") {
    val app = newApp()
    val res = post(app, "/tasks", "Buy milk")
    assertEquals(res.status, Status.Created)
    val task = res.as[Task].unsafeRunSync()
    assertEquals(task.id, 1L)
    assertEquals(task.title, "Buy milk")
    assertEquals(task.done, false)
  }

  test("GET /tasks/1 after creation returns the task") {
    val app = newApp()
    post(app, "/tasks", "Buy milk")
    val res = get(app, "/tasks/1")
    assertEquals(res.status, Status.Ok)
    val task = res.as[Task].unsafeRunSync()
    assertEquals(task.id, 1L)
    assertEquals(task.title, "Buy milk")
  }

  test("GET /tasks/999 returns 404") {
    val app = newApp()
    val res = get(app, "/tasks/999")
    assertEquals(res.status, Status.NotFound)
  }

  test("DELETE an existing task returns 204, and a subsequent GET returns 404") {
    val app = newApp()
    post(app, "/tasks", "Buy milk")
    val del = app.run(Request[IO](Method.DELETE, Uri.unsafeFromString("/tasks/1"))).unsafeRunSync()
    assertEquals(del.status, Status.NoContent)
    val res = get(app, "/tasks/1")
    assertEquals(res.status, Status.NotFound)
  }
}
