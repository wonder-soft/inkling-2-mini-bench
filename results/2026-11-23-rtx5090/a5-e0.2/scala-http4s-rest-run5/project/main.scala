//> using scala 3.3.4
//> using dep org.http4s::http4s-ember-server:0.23.30
//> using dep org.http4s::http4s-dsl:0.23.30
//> using dep org.http4s::http4s-circe:0.23.30
//> using dep io.circe::circe-generic:0.14.10
//> using test.dep org.scalameta::munit::1.0.4

import cats.effect.{IO, IOApp, Ref}
import org.http4s._
import org.http4s.dsl.io._
import org.http4s.implicits._
import org.http4s.circe._
import org.http4s.ember.server.EmberServerBuilder
import io.circe._
import io.circe.generic.auto._
import io.circe.syntax._

case class Task(id: Long, title: String, done: Boolean)

object Task {
  implicit val encoder: Encoder[Task] = deriveEncoder[Task]
  implicit val decoder: Decoder[Task] = deriveDecoder[Task]
}

case class CreateTask(title: String)
object CreateTask {
  implicit val decoder: Decoder[CreateTask] = deriveDecoder[CreateTask]
}

case class UpdateTask(title: String, done: Boolean)
object UpdateTask {
  implicit val decoder: Decoder[UpdateTask] = deriveDecoder[UpdateTask]
}

def routes(store: Ref[IO, Map[Long, Task]], counter: Ref[IO, Long]): HttpRoutes[IO] =
  HttpRoutes.of[IO] {
    case GET -> Root / "health" =>
      Ok(json"""{"status":"ok"}""")

    case GET -> Root / "tasks" =>
      store.get.map(_.values.toList.sortBy(_.id)).map(_.asJson).flatMap(Ok(_))

    case req @ POST -> Root / "tasks" =>
      req.as[CreateTask].flatMap { ct =>
        counter.modify(n => (n + 1, n + 1)).flatMap { id =>
          val task = Task(id, ct.title, done = false)
          store.update(_ + (id -> task)).as(Created(task.asJson))
        }
      }

    case GET -> Root / "tasks" / LongVar(id) =>
      store.get.flatMap { m =>
        m.get(id) match {
          case Some(t) => Ok(t.asJson)
          case None     => NotFound()
        }
      }

    case req @ PUT -> Root / "tasks" / LongVar(id) =>
      req.as[UpdateTask].flatMap { ut =>
        store.get.flatMap { m =>
          m.get(id) match {
            case Some(t) =>
              val nt = t.copy(title = ut.title, done = ut.done)
              store.set(m.updated(id, nt)).as(Ok(nt.asJson))
            case None =>
              NotFound()
          }
        }
      }

    case DELETE -> Root / "tasks" / LongVar(id) =>
      store.get.flatMap { m =>
        m.get(id) match {
          case Some(_) => store.update(_ - id).as(NoContent())
          case None     => NotFound()
        }
      }
  }

def freshApp: IO[HttpApp[IO]] =
  for {
    store   <- Ref.of[IO, Map[Long, Task]](Map.empty)
    counter <- Ref.of[IO, Long](0L)
  } yield routes(store, counter)

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
}
