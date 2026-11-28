//> using scala 3.3.4
//> using dep org.http4s::http4s-ember-server:0.23.30
//> using dep org.http4s::http4s-dsl:0.23.30
//> using dep org.http4s::http4s-circe:0.23.30
//> using dep io.circe::circe-generic:0.14.10
//> using test.dep org.scalameta::munit::1.0.4

import cats.effect.{IO, IOApp, Ref}
import org.http4s._
import org.http4s.implicits._
import org.http4s.dsl.io._
import org.http4s.circe._
import org.http4s.ember.server.EmberServerBuilder
import io.circe.generic.auto._
import io.circe.syntax._

case class Task(id: Long, title: String, done: Boolean)
case class CreateTask(title: String)
case class UpdateTask(title: String, done: Boolean)

def routes(store: Ref[IO, List[Task]], counter: Ref[IO, Long]): HttpRoutes[IO] =
  HttpRoutes.of[IO] {
    case GET -> Root / "health" =>
      Ok(Json.obj("status" -> "ok".asJson))

    case GET -> Root / "tasks" =>
      store.get.flatMap { tasks =>
        Ok(tasks.sortBy(_.id).asJson)
      }

    case req @ POST -> Root / "tasks" =>
      req.as[CreateTask].flatMap { body =>
        counter.modify(n => (n, n + 1)).flatMap { id =>
          val task = Task(id, body.title, done = false)
          store.update(_ :+ task).flatMap { _ =>
            IO.pure(Created(task.asJson))
          }
        }
      }

    case GET -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case None => NotFound()
        case Some(id) =>
          store.get.flatMap { tasks =>
            tasks.find(_.id == id) match {
              case Some(task) => Ok(task.asJson)
              case None       => NotFound()
            }
          }
      }

    case req @ PUT -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case None => NotFound()
        case Some(id) =>
          req.as[UpdateTask].flatMap { body =>
            store.modify { tasks =>
              tasks.find(_.id == id) match {
                case Some(existing) =>
                  val updated = existing.copy(title = body.title, done = body.done)
                  (Right(updated), tasks.map(t => if (t.id == id) updated else t))
                case None =>
                  (Left(()), tasks)
              }
            }.flatMap {
              case Right(updated) => Ok(updated.asJson)
              case Left(_)        => NotFound()
            }
          }
      }

    case DELETE -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case None => NotFound()
        case Some(id) =>
          store.modify { tasks =>
            val found = tasks.exists(_.id == id)
            (found, tasks.filterNot(_.id == id))
          }.flatMap {
            case true  => NoContent()
            case false => NotFound()
          }
      }
  }

def freshApp: IO[HttpApp[IO]] =
  Ref[IO].of(List.empty[Task]).flatMap { store =>
    Ref[IO].of(1L).map { counter =>
      routes(store, counter).orNotFound
    }
  }

object Main extends IOApp.Simple {
  override def run: IO[Unit] =
    freshApp.flatMap { app =>
      EmberServerBuilder.default[IO]
        .withHost(host"0.0.0.0")
        .withPort(port"3000")
        .build(app)
        .useForever
    }
}
