import * as ui from "@:golem/ui";
import { sql } from "./database";

let loggedInUser: User | null = null;

export const DEFAULT_USERNAME = "admin";

export interface UserRow {
  id: number;
  username: string;
  password: string | null;
  createdAt: Date;
  admin: boolean;
}

export class User {
  /**
   * Convert a password from a prompt into a string.
   */
  public static passwordToString(password: string[] | null): string | null {
    if (password === null) {
      return null;
    }

    return `[${password.join(", ")}]`;
  }

  /**
   * Get the currently logged-in user.
   * @returns The logged-in user, or `null` if no user is logged in.
   */
  public static async loggedInUser(): Promise<User | null> {
    return loggedInUser;
  }

  /**
   * Log in a user.
   * @param username The username of the user to login.
   * @param force Whether to force the login, even if the user has a password.
   * @returns The logged-in user, or `null` if the user could not be logged in
   *          (e.g. invalid password).
   */
  public static async login(
    username: string,
    force = false,
  ): Promise<User | null> {
    let [user] = await sql<UserRow>`SELECT *
                                        FROM users
                                        WHERE username = ${username}`;

    if (!user) {
      throw new Error("Invalid username or password");
    }

    if (!force && user.password !== null) {
      let prompt = "";
      while (true) {
        const password = await ui.promptPassword(
          "Enter your password:",
          prompt,
          4,
        );
        if (password === null) {
          return null;
        }

        if (this.passwordToString(password) === user.password) {
          break;
        }

        prompt = "Invalid password. Please try again:";
      }
    }

    loggedInUser = new this(+user.id, "" + user.username, user.admin);
    return loggedInUser;
  }

  /**
   * Create a new user. Please note that the user is not logged in after creation.
   * @param username The username of the new user.
   * @param password The password of the new user, or `null` if the user should
   *                 not have a password.
   * @param admin Whether the new user should be an admin.
   * @returns The newly created user.
   * @throws If the user already exists or there is a problem adding the user.
   */
  public static async create(
    username: string,
    password: string[] | null,
    admin: boolean,
  ): Promise<void> {
    await sql`INSERT INTO users ${sql.insertValues({
      username,
      password: this.passwordToString(password),
      admin,
    })}`;
  }

  private constructor(
    public readonly id: number,
    public readonly username: string,
    public readonly admin: boolean,
  ) {}
}
