import logging
import subprocess


def parse():
    with open("desired_output.txt") as fi:
        with open("parsed_desired_output.txt", "w") as fo:
            for line in fi.readlines():
                opcode = line[0:4]
                status_start = line.find("A:")
                status_end = line.find("SP") + 5
                output_line = "{} {}\n".format(opcode, line[status_start: status_end])
                fo.write(output_line)


def compare():
    result = []
    with open("output.txt") as actual:
        with open("parsed_desired_output.txt") as expected:
            counter = 0
            for actual_line, expected_line in zip(actual.readlines(), expected.readlines()):
                counter += 1
                if not actual_line:
                    logging.warning("Missing line {}".format(counter))
                    logging.warning(result)
                    return
                if actual_line != expected_line:
                    result.append(counter)
    logging.warning(result)


if __name__ == "__main__":
    # subprocess.run(["../cargo run --package r_nes --bin r_nes"])
    parse()
    compare()
