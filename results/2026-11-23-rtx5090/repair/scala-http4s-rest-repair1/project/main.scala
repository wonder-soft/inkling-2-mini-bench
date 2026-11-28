//> using scala 3.3.4
//> using dep org.http4s::http4s-ember-server:0.23.30
//> using dep org.http4s::http4s-dsl:0.23.30
//> using dep org.http4s::http4s-circe:0.23.30
//> using dep io.circe::circe-generic:0.14.10
//> using test.dep org.scalameta::munit::1.0.4

import cats.effect.{IO, Ref, IOApp}
import org.http4s.{HttpApp, HttpRoutes, Request, Status}
import org.http4s.dsl.io._
import org.http4s.circe.CirceEntityCodec._
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.{host, port}
import io.circe.generic.auto._
import io.circe.syntax._

case class Task(id: Long, title: String, done: Boolean)
case class NewTask(title: String)
case class UpdateTask(title: String, done: Boolean)

def routes(store: Ref[IO, Map[Long, Task]], counter: Ref[IO, Long]): HttpRoutes[IO] =
  HttpRoutes.of[IO] {
    case GET -> Root / "health" =>
      Ok(io.circe.Json.obj("status" -> io.circe.Json.fromString("ok")))

    case GET -> Root / "tasks" =>
      store.get.flatMap { s =>
        Ok(s.toList.sortBy(_._1).map(_._2))
      }

    case req @ POST -> Root / "tasks" =>
      req.as[NewTask].flatMap { nt =>
        counter.updateAndGet(_ + 1).flatMap { id =>
          val task = Task(id, nt.title, done = false)
          store.update(_ + (id -> task)).as(task)
        }.flatMap(task => Created(task))
      }

    case GET -> Root / "tasks" / LongVar(id) =>
      store.get.flatMap { s =>
        s.get(id) match {
          case Some(t) => Ok(t)
          case None     => NotFound()
        }
      }

    case req @ PUT -> Root / "tasks" / LongVar(id) =>
      req.as[UpdateTask].flatMap { ut =>
        store.modify { s =>
          s.get(id) match {
            case Some(t) =>
              val updated = t.copy(title = ut.title, done = ut.done)
              (s.updated(id, updated), Some(updated))
            case None =>
              (s, None)
          }
        }.flatMap {
          case Some(t) => Ok(t)
          case None    => NotFound()
        }
      }

    case DELETE -> Root / "tasks" / LongVar(id) =>
      store.modify { s =>
        if (s.contains(id)) (s - id, true) else (s, false)
      }.flatMap {
        case true  => NoContent()
        case false => NotFound()
      }
  }

def freshApp: IO[HttpApp[IO]] =
  for {
    store   <- Ref.of[IO, Map[Long, Task]](Map.empty)
    counter <- Ref.of[IO, Long](0L)
  } yield routes(store, counter).orNotFound

object Main extends IOApp.Simple {
  def run: IO[Unit] =
    freshApp.flatMap { app =>
      EmberServerBuilder.default[IO]
        .withHost(host"0.0.0.0")
        .withPort(port"3000")
        .withHttpApp(app)
        .build
        .useForever
    }
}
