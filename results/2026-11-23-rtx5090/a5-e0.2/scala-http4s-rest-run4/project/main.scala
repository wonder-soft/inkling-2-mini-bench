//> using scala 3.3.4
//> using dep org.http4s::http4s-ember-server:0.23.30
//> using dep org.http4s::http4s-dsl:0.23.30
//> using dep org.http4s::http4s-circe:0.23.30
//> using dep io.circe::circe-generic:0.14.10
//> using test.dep org.scalameta::munit::1.0.4

import cats.effect.{IO, Ref, IOApp}
import cats.syntax.all.*
import io.circe.{Json, generic}
import org.http4s.*
import org.http4s.circe.*
import org.http4s.dsl.io.*
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.implicits.*

case class Task(id: Long, title: String, done: Boolean)

case class CreateTask(title: String)
case class UpdateTask(title: String, done: Boolean)

given io.circe.Encoder[Task] = generic.deriveEncoder[Task]
given io.circe.Decoder[Task] = generic.deriveDecoder[Task]
given io.circe.Encoder[CreateTask] = generic.deriveEncoder[CreateTask]
given io.circe.Decoder[CreateTask] = generic.deriveDecoder[CreateTask]
given io.circe.Encoder[UpdateTask] = generic.deriveEncoder[UpdateTask]
given io.circe.Decoder[UpdateTask] = generic.deriveDecoder[UpdateTask]

def routes(store: Ref[IO, Map[Long, Task]], counter: Ref[IO, Long]): HttpRoutes[IO] =
  HttpRoutes.of[IO] {
    case GET -> Root / "health" =>
      Ok(Json.obj("status" -> Json.fromString("ok")))

    case GET -> Root / "tasks" =>
      store.get.flatMap { m =>
        Ok(m.values.toList.sortBy(_.id))
      }

    case req @ POST -> Root / "tasks" =>
      req.as[CreateTask].flatMap { body =>
        counter.modify(n => (n + 1, n + 1)).flatMap { id =>
          val task = Task(id, body.title, done = false)
          store.update(_ + (id -> task)).as(Created(task))
        }
      }

    case GET -> Root / "tasks" / LongVar(id) =>
      store.get.flatMap { m =>
        m.get(id) match {
          case Some(task) => Ok(task)
          case None       => NotFound()
        }
      }

    case req @ PUT -> Root / "tasks" / LongVar(id) =>
      req.as[UpdateTask].flatMap { body =>
        store.get.flatMap { m =>
          m.get(id) match {
            case Some(_) =>
              val updated = Task(id, body.title, body.done)
              store.set(m.updated(id, updated)).as(Ok(updated))
            case None => NotFound()
          }
        }
      }

    case DELETE -> Root / "tasks" / LongVar(id) =>
      store.get.flatMap { m =>
        m.get(id) match {
          case Some(_) => store.set(m - id).as(NoContent())
          case None    => NotFound()
        }
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
      EmberServerBuilder
        .default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"3000")
        .withHttpApp(app)
        .build
        .useForever
    }
}
