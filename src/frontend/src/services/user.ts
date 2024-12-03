import * as osd from "1fpga:osd";
import { sql } from "$/utils";

let loggedInUser: User | null = null;

export const DEFAULT_USERNAME = "admin";

export interface UserRow {
  id: number;
  username: string;
  password: string | null;
  createdAt: Date;
  admin: number;
}

export class User {
  /**
   * Log out the currently logged-in user.
   */
  public static async logout() {
    loggedInUser = null;
  }

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
   * @param fail Whether to throw an error if no user is logged in.
   * @returns The logged-in user, or `null` if no user is logged in.
   */
  public static loggedInUser(fail: true): User;
  public static loggedInUser(fail?: boolean): User | null {
    if (loggedInUser === null && fail) {
      throw new Error("No user logged in");
    }
    return loggedInUser;
  }

  public static async list(): Promise<User[]> {
    const rows = await sql<UserRow>`SELECT *
                                    FROM users`;
    return rows.map((row) => new this(row.id, row.username, !!row.admin));
  }

  public static async byUsername(username: string): Promise<User | null> {
    let [user] = await sql<UserRow>`SELECT *
                                    FROM users
                                    WHERE username = ${username}`;
    if (!user) {
      return null;
    }
    return new this(user.id, user.username, !!user.admin);
  }

  /**
   * Log in a user.
   * @param username The username of the user to login.
   * @param force Whether to force the login, even if the user has a password.
   * @returns The logged-in user, or `null` if the user could not be logged in
   *          (e.g. invalid password).
   */
  public static async login(
    username: string = DEFAULT_USERNAME,
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
        const password = await osd.promptPassword(
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

    loggedInUser = new this(+user.id, "" + user.username, !!user.admin);
    return loggedInUser;
  }

  /**
   * Return whether the user is able to logout. Users can logout if there is
   * more than one user, or if the user has a password.
   */
  static async canLogOut() {
    const [{ count }] = await sql<{ count: number }>`SELECT COUNT(*) as count
                                                     FROM users`;

    if (count > 1) {
      return true;
    }
    if (loggedInUser === null) {
      return false;
    }

    let [user] = await sql<UserRow>`SELECT *
                                    FROM users
                                    WHERE username = ${loggedInUser.username}`;

    if (!user) {
      return false;
    }
    return user.password !== null;
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
    private admin_: boolean,
  ) {}

  public get admin() {
    return this.admin_;
  }

  public async delete() {
    if (loggedInUser?.admin !== true) {
      throw new Error("You do not have permission to delete users");
    }

    await sql`DELETE
              FROM users
              WHERE id = ${this.id}`;
  }

  public async clearPassword() {
    await sql`UPDATE users
              SET password = null
              WHERE id = ${this.id}`;
  }

  public async setPassword(password: string[]) {
    await sql`UPDATE users
              SET password = ${User.passwordToString(password)}
              WHERE id = ${this.id}`;
  }

  public async toggleAdmin() {
    if (loggedInUser?.admin) {
      throw new Error("You do not have permission to change user admin status");
    }

    this.admin_ = !this.admin;
    await sql`UPDATE users
              SET admin = ${this.admin}
              WHERE id = ${this.id}`;
  }
}
