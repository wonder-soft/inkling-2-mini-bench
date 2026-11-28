//> using scala 3.3.4
//> using dep org.http4s::http4s-ember-server:0.23.30
//> using dep org.http4s::http4s-dsl:0.23.30
//> using dep org.http4s::http4s-circe:0.23.30
//> using dep io.circe::circe-generic:0.14.10
//> using test.dep org.scalameta::munit::1.0.4

import cats.effect.{IO, IOApp, Ref}
import io.circe.generic.auto._
import io.circe.syntax._
import org.http4s._
import org.http4s.circe._
import org.http4s.dsl.io._
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.implicits._

case class Task(id: Long, title: String, done: Boolean)
case class CreateTaskBody(title: String)
case class UpdateTaskBody(title: String, done: Boolean)

def routes(store: Ref[IO, Map[Long, Task]], counter: Ref[IO, Long]): HttpRoutes[IO] =
  HttpRoutes.of[IO] {
    case GET -> Root / "health" =>
      Ok(Json.obj("status" -> "ok".asJson))

    case GET -> Root / "tasks" =>
      store.get.map { tasks =>
        val ordered = tasks.values.toList.sortBy(_.id)
        Response[IO](Status.Ok).withEntity(ordered.asJson)
      }

    case req @ POST -> Root / "tasks" =>
      req.as[CreateTaskBody].flatMap { body =>
        counter.modify(n => (n + 1, n + 1)).flatMap { id =>
          val task = Task(id, body.title, done = false)
          store.update(_ + (id -> task)).as(Response[IO](Status.Created).withEntity(task.asJson))
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
        case None => NotFound()
      }

    case req @ PUT -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case Some(id) =>
          req.as[UpdateTaskBody].flatMap { body =>
            store.modify { tasks =>
              tasks.get(id) match {
                case Some(old) =>
                  val updated = old.copy(title = body.title, done = body.done)
                  (updated, tasks + (id -> updated))
                case None =>
                  (None, tasks)
              }
            }.flatMap {
              case Some(updated) => Ok(updated)
              case None => NotFound()
            }
          }
        case None => NotFound()
      }

    case DELETE -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case Some(id) =>
          store.modify { tasks =>
            if (tasks.contains(id)) (true, tasks - id)
            else (false, tasks)
          }.flatMap {
            case true => Response[IO](Status.NoContent)
            case false => NotFound()
          }
        case None => NotFound()
      }
  }

def freshApp: IO[HttpApp[IO]] =
  IO.bothRef(Ref.of[IO, Map[Long, Task]](Map.empty), Ref.of[IO, Long](0L)).map {
    case (store, counter) => routes(store, counter).orNotFound
  }

object Main extends IOApp.Simple {
  def run: IO[Unit] =
    freshApp.flatMap { app =>
      EmberServerBuilder.default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"3000")
        .withHttpApp(app)
        .build
        .useForever
    }
