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
      Ok(Json.obj("status" -> "ok".asJson))

    case GET -> Root / "tasks" =>
      store.get.flatMap { m =>
        Ok(m.values.toList.sortBy(_.id).asJson)
      }

    case req @ POST -> Root / "tasks" =>
      req.as[CreateTask].attempt.flatMap {
        case Left(_) =>
          BadRequest(Json.obj("error" -> "invalid body".asJson))
        case Right(ct) =>
          counter.updateAndGet(_ + 1).flatMap { id =>
            val task = Task(id, ct.title, done = false)
            store.update(_ + (id -> task)).as(task)
          }.flatMap(t => Created(t.asJson))
      }

    case GET -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case None => NotFound()
        case Some(id) =>
          store.get.flatMap { m =>
            m.get(id) match {
              case Some(t) => Ok(t.asJson)
              case None => NotFound()
            }
          }
      }

    case req @ PUT -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case None => NotFound()
        case Some(id) =>
          req.as[UpdateTask].attempt.flatMap {
            case Left(_) =>
              BadRequest(Json.obj("error" -> "invalid body".asJson))
            case Right(ut) =>
              store.modify { m =>
                m.get(id) match {
                  case Some(t) =>
                    val updated = t.copy(title = ut.title, done = ut.done)
                    (Some(updated), m + (id -> updated))
                  case None =>
                    (None, m)
                }
              }.flatMap {
                case Some(t) => Ok(t.asJson)
                case None => NotFound()
              }
          }
      }

    case DELETE -> Root / "tasks" / idStr =>
      idStr.toLongOption match {
        case None => NotFound()
        case Some(id) =>
          store.modify { m =>
            m.get(id) match {
              case Some(_) => (true, m - id)
              case None => (false, m)
            }
          }.flatMap {
            case true => NoContent()
            case false => NotFound()
          }
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
        .build(app)
        .useForever
    }
}
