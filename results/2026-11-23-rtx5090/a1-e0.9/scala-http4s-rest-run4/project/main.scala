//> using scala 3.3.4
//> using dep org.http4s::http4s-ember-server:0.23.30
//> using dep org.http4s::http4s-dsl:0.23.30
//> using dep org.http4s::http4s-circe:0.23.30
//> using dep io.circe::circe-generic:0.14.10
//> using test.dep org.scalameta::munit::1.0.4

import cats.effect.{IO, IOApp, Ref}
import io.circe.Json
import io.circe.generic.auto._
import io.circe.syntax._
import org.http4s._
import org.http4s.circe._
import org.http4s.dsl.io._
import org.http4s.ember.server.EmberServerBuilder

case class Task(id: Long, title: String, done: Boolean)
case class CreateTask(title: String)
case class UpdateTask(title: String, done: Boolean)

def routes(store: Ref[IO, Map[Long, Task]], counter: Ref[IO, Long]): HttpRoutes[IO] =
  HttpRoutes.of[IO] {
    case GET -> Root / "health" =>
      Ok(Json.obj("status" -> Json.fromString("ok")))

    case GET -> Root / "tasks" =>
      store.get.flatMap { tasks =>
        Ok(tasks.values.toList.sortBy(_.id))
      }

    case req @ POST -> Root / "tasks" =>
      req.as[CreateTask].flatMap { body =>
        counter.modify { n => (n + 1L, n + 1L) }.flatMap { newId =>
          val task = Task(newId, body.title, done = false)
          store.update(_ + (newId -> task)).flatMap(_ => Created(task))
        }
      }

    case GET -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case Some(id) =>
          store.get.flatMap { tasks =>
            tasks.get(id) match {
              case Some(task) => Ok(task)
              case None => NotFound()
            }
          }
        case None =>
          NotFound()
      }

    case req @ PUT -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case Some(id) =>
          req.as[UpdateTask].flatMap { body =>
            store.get.flatMap { tasks =>
              tasks.get(id) match {
                case Some(task) =>
                  val updated = task.copy(title = body.title, done = body.done)
                  store.update(_ + (id -> updated)).flatMap(_ => Ok(updated))
                case None => NotFound()
              }
            }
          }
        case None => NotFound()
      }

    case DELETE -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case Some(id) =>
          store.get.flatMap { tasks =>
            if (tasks.contains(id)) {
              store.update(_ - id).flatMap(_ => NoContent())
            } else {
              NotFound()
            }
          }
        case None => NotFound()
      }
  }

def freshApp: IO[HttpApp[IO]] =
  for {
    store <- Ref.of[IO, Map[Long, Task]](Map.empty)
    counter <- Ref.of[IO, Long](0L)
  } yield routes(store, counter).orNotFound

object Main extends IOApp.Simple {
  def run: IO[Unit] =
    freshApp.flatMap { app =>
      EmberServerBuilder.default[IO]
        .withPort(3000)
        .withHttpApp(app)
        .build
        .useForever
    }
}
